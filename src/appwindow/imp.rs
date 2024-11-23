// Imports
use crate::{config, dialogs, RnMainHeader, RnOverlays, RnSidebar};
use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    gdk, glib, glib::clone, CompositeTemplate, CssProvider, PackType, PadController,
};
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Debug, CompositeTemplate)]
#[template(resource = "/com/github/linruohan/mytool/ui/appwindow.ui")]
pub(crate) struct RnAppWindow {
    pub(crate) drawing_pad_controller: RefCell<Option<PadController>>,
    pub(crate) autosave_source_id: RefCell<Option<glib::SourceId>>,
    pub(crate) periodic_configsave_source_id: RefCell<Option<glib::SourceId>>,

    pub(crate) save_in_progress: Cell<bool>,
    pub(crate) save_in_progress_toast: RefCell<Option<adw::Toast>>,
    pub(crate) autosave: Cell<bool>,
    pub(crate) autosave_interval_secs: Cell<u32>,
    pub(crate) righthanded: Cell<bool>,
    pub(crate) block_pinch_zoom: Cell<bool>,
    pub(crate) respect_borders: Cell<bool>,
    pub(crate) close_in_progress: Cell<bool>,

    #[template_child]
    pub(crate) overview: TemplateChild<adw::TabOverview>,
    #[template_child]
    pub(crate) main_header: TemplateChild<RnMainHeader>,
    #[template_child]
    pub(crate) split_view: TemplateChild<adw::OverlaySplitView>,
    #[template_child]
    pub(crate) sidebar: TemplateChild<RnSidebar>,
    #[template_child]
    pub(crate) tabbar: TemplateChild<adw::TabBar>,
    #[template_child]
    pub(crate) overlays: TemplateChild<RnOverlays>,
}

impl Default for RnAppWindow {
    fn default() -> Self {
        Self {
            drawing_pad_controller: RefCell::new(None),
            autosave_source_id: RefCell::new(None),
            periodic_configsave_source_id: RefCell::new(None),

            save_in_progress: Cell::new(false),
            save_in_progress_toast: RefCell::new(None),
            autosave: Cell::new(true),
            autosave_interval_secs: Cell::new(
                super::RnAppWindow::AUTOSAVE_INTERVAL_DEFAULT,
            ),
            righthanded: Cell::new(true),
            block_pinch_zoom: Cell::new(false),
            respect_borders: Cell::new(false),
            close_in_progress: Cell::new(false),

            overview: TemplateChild::<adw::TabOverview>::default(),
            main_header: TemplateChild::<RnMainHeader>::default(),
            split_view: TemplateChild::<adw::OverlaySplitView>::default(),
            sidebar: TemplateChild::<RnSidebar>::default(),
            tabbar: TemplateChild::<adw::TabBar>::default(),
            overlays: TemplateChild::<RnOverlays>::default(),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for RnAppWindow {
    const NAME: &'static str = "RnAppWindow";
    type Type = super::RnAppWindow;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for RnAppWindow {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        let _windowsettings = obj.settings();

        if config::PROFILE == "devel" {
            obj.add_css_class("devel");
        }

        // Load the application css
        let css = CssProvider::new();
        css.load_from_resource(
            (String::from(config::APP_IDPATH) + "ui/style.css").as_str(),
        );

        let display = gdk::Display::default().unwrap();
        gtk::style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        self.setup_input();
        self.setup_overview();
        self.setup_split_view();
        self.setup_tabbar();
    }

    fn dispose(&self) {
        self.dispose_template();
        while let Some(child) = self.obj().first_child() {
            child.unparent();
        }
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecBoolean::builder("save-in-progress")
                    .default_value(false)
                    .build(),
                glib::ParamSpecBoolean::builder("autosave")
                    .default_value(false)
                    .build(),
                glib::ParamSpecUInt::builder("autosave-interval-secs")
                    .minimum(5)
                    .maximum(u32::MAX)
                    .default_value(super::RnAppWindow::AUTOSAVE_INTERVAL_DEFAULT)
                    .build(),
                glib::ParamSpecBoolean::builder("righthanded")
                    .default_value(false)
                    .build(),
                glib::ParamSpecBoolean::builder("block-pinch-zoom")
                    .default_value(false)
                    .build(),
                glib::ParamSpecBoolean::builder("respect-borders")
                    .default_value(false)
                    .build(),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "save-in-progress" => self.save_in_progress.get().to_value(),
            "autosave" => self.autosave.get().to_value(),
            "autosave-interval-secs" => self.autosave_interval_secs.get().to_value(),
            "righthanded" => self.righthanded.get().to_value(),
            "block-pinch-zoom" => self.block_pinch_zoom.get().to_value(),
            "respect-borders" => self.respect_borders.get().to_value(),
            _ => unimplemented!(),
        }
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "save-in-progress" => {
                let save_in_progress = value
                    .get::<bool>()
                    .expect("The value needs to be of type `bool`");
                self.save_in_progress.replace(save_in_progress);
            }
            "autosave" => {
                let autosave = value
                    .get::<bool>()
                    .expect("The value needs to be of type `bool`");

                self.autosave.replace(autosave);

                if autosave {
                    self.update_autosave_handler();
                } else if let Some(autosave_source_id) =
                    self.autosave_source_id.borrow_mut().take()
                {
                    autosave_source_id.remove();
                }
            }
            "autosave-interval-secs" => {
                let autosave_interval_secs = value
                    .get::<u32>()
                    .expect("The value needs to be of type `u32`");

                self.autosave_interval_secs.replace(autosave_interval_secs);

                if self.autosave.get() {
                    self.update_autosave_handler();
                }
            }
            "righthanded" => {
                let righthanded = value
                    .get::<bool>()
                    .expect("The value needs to be of type `bool`");

                self.righthanded.replace(righthanded);

                self.handle_righthanded_property(righthanded);
            }
            "block-pinch-zoom" => {
                let block_pinch_zoom: bool =
                    value.get().expect("The value needs to be of type `bool`");
                self.block_pinch_zoom.replace(block_pinch_zoom);
            }
            "respect-borders" => {
                let respect_borders: bool =
                    value.get().expect("The value needs to be of type `bool`");
                self.respect_borders.replace(respect_borders);
            }
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for RnAppWindow {}

impl WindowImpl for RnAppWindow {
    fn close_request(&self) -> glib::Propagation {
        let obj = self.obj().to_owned();
        if self.close_in_progress.get() {
            return glib::Propagation::Stop;
        }

        if obj.tabs_any_saves_in_progress() {
            obj.connect_notify_local(Some("save-in-progress"), move |appwindow, _| {
                if !appwindow.save_in_progress() {
                    if appwindow.tabs_any_unsaved_changes() {
                        appwindow.imp().close_in_progress.set(false);
                        appwindow.main_header().headerbar().set_sensitive(true);
                        appwindow.sidebar().headerbar().set_sensitive(true);
                        if let Some(toast) =
                            appwindow.imp().save_in_progress_toast.take()
                        {
                            toast.dismiss();
                        }

                        glib::spawn_future_local(clone!(
                            #[weak]
                            appwindow,
                            async move {
                                dialogs::dialog_close_window(&appwindow).await;
                            }
                        ));
                    } else {
                        appwindow.close_force();
                    }
                }
            });
            self.close_in_progress.set(true);
            self.main_header.headerbar().set_sensitive(false);
            self.sidebar.headerbar().set_sensitive(false);
            // obj.overlays().dispatch_toast_text_singleton(
            //     &gettext("Saves are in progress, waiting before closing.."),
            //     None,
            //     &mut self.save_in_progress_toast.borrow_mut(),
            // );
        } else if obj.tabs_any_unsaved_changes() {
            glib::spawn_future_local(clone!(
                #[weak(rename_to=appwindow)]
                obj,
                async move {
                    dialogs::dialog_close_window(&appwindow).await;
                }
            ));
        } else {
            obj.close_force();
        }

        // Inhibit (Overwrite) the default handler. This handler is then responsible for destroying the window.
        glib::Propagation::Stop
    }
}

impl ApplicationWindowImpl for RnAppWindow {}
impl AdwWindowImpl for RnAppWindow {}
impl AdwApplicationWindowImpl for RnAppWindow {}

impl RnAppWindow {
    fn update_autosave_handler(&self) {
        let obj = self.obj();

        if let Some(removed_id) = self.autosave_source_id.borrow_mut().replace(
            glib::source::timeout_add_seconds_local(
                self.autosave_interval_secs.get(),
                clone!(
                    #[weak(rename_to=appwindow)]
                    obj,
                    #[upgrade_or]
                    glib::ControlFlow::Break,
                    move || {
                        // save for all tabs opened in the current window that have unsaved changes
                        let tabs = appwindow.get_all_tabs();

                        for (i, tab) in tabs.iter().enumerate() {}

                        glib::ControlFlow::Continue
                    }
                ),
            ),
        ) {
            removed_id.remove();
        }
    }

    fn setup_input(&self) {
        let obj = self.obj();
        let drawing_pad_controller = PadController::new(&*obj, None);

        // drawing_pad_controller.set_action(
        //     PadActionType::Button,
        //     0,
        //     -1,
        //     &gettext("Button 1"),
        //     "drawing-pad-pressed-button-0",
        // );
        // drawing_pad_controller.set_action(
        //     PadActionType::Button,
        //     1,
        //     -1,
        //     &gettext("Button 2"),
        //     "drawing-pad-pressed-button-1",
        // );
        // drawing_pad_controller.set_action(
        //     PadActionType::Button,
        //     2,
        //     -1,
        //     &gettext("Button 3"),
        //     "drawing-pad-pressed-button-2",
        // );
        // drawing_pad_controller.set_action(
        //     PadActionType::Button,
        //     3,
        //     -1,
        //     &gettext("Button 4"),
        //     "drawing-pad-pressed-button-3",
        // );

        obj.add_controller(drawing_pad_controller.clone());
        self.drawing_pad_controller
            .replace(Some(drawing_pad_controller));
    }

    fn setup_overview(&self) {
        self.overview.set_view(Some(&self.overlays.tabview()));

        let obj = self.obj();

        // Create new tab via tab overview
        self.overview.connect_create_tab(clone!(
            #[weak(rename_to=appwindow)]
            obj,
            #[upgrade_or_panic]
            move |_| {
                let wrapper = appwindow.new_canvas_wrapper();
                appwindow.append_wrapper_new_tab(&wrapper)
            }
        ));
    }

    fn setup_tabbar(&self) {
        self.tabbar.set_view(Some(&self.overlays.tabview()));
    }

    fn setup_split_view(&self) {
        let obj = self.obj();
        let split_view = self.split_view.get();
        let left_sidebar_reveal_toggle = obj.main_header().left_sidebar_reveal_toggle();
        let right_sidebar_reveal_toggle =
            obj.main_header().right_sidebar_reveal_toggle();

        left_sidebar_reveal_toggle
            .bind_property("active", &right_sidebar_reveal_toggle, "active")
            .sync_create()
            .bidirectional()
            .build();

        left_sidebar_reveal_toggle
            .bind_property("active", &split_view, "show-sidebar")
            .sync_create()
            .bidirectional()
            .build();
        right_sidebar_reveal_toggle
            .bind_property("active", &split_view, "show-sidebar")
            .sync_create()
            .bidirectional()
            .build();

        let update_widgets =
            move |split_view: &adw::OverlaySplitView,
                  appwindow: &super::RnAppWindow| {
                let sidebar_position = split_view.sidebar_position();
                let sidebar_collapsed = split_view.is_collapsed();
                let sidebar_shown = split_view.shows_sidebar();

                let sidebar_appmenu_visibility = !sidebar_collapsed && sidebar_shown;
                let sidebar_left_close_button_visibility = (sidebar_position
                    == PackType::End)
                    && sidebar_collapsed
                    && sidebar_shown;
                let sidebar_right_close_button_visibility = (sidebar_position
                    == PackType::Start)
                    && sidebar_collapsed
                    && sidebar_shown;

                appwindow
                    .main_header()
                    .appmenu()
                    .set_visible(!sidebar_appmenu_visibility);
                appwindow
                    .sidebar()
                    .appmenu()
                    .set_visible(sidebar_appmenu_visibility);
                appwindow
                    .sidebar()
                    .left_close_button()
                    .set_visible(sidebar_left_close_button_visibility);
                appwindow
                    .sidebar()
                    .right_close_button()
                    .set_visible(sidebar_right_close_button_visibility);

                if sidebar_position == PackType::End {
                    appwindow
                        .sidebar()
                        .left_close_button()
                        .set_icon_name("dir-right-symbolic");
                    appwindow
                        .sidebar()
                        .right_close_button()
                        .set_icon_name("dir-right-symbolic");
                } else {
                    appwindow
                        .sidebar()
                        .left_close_button()
                        .set_icon_name("dir-left-symbolic");
                    appwindow
                        .sidebar()
                        .right_close_button()
                        .set_icon_name("dir-left-symbolic");
                }
            };

        let sidebar_expanded_shown = Rc::new(Cell::new(false));

        self.split_view.connect_show_sidebar_notify(clone!(
            #[strong]
            sidebar_expanded_shown,
            #[weak(rename_to=appwindow)]
            obj,
            move |split_view| {
                if !split_view.is_collapsed() {
                    sidebar_expanded_shown.set(split_view.shows_sidebar());
                }
                update_widgets(split_view, &appwindow);
            }
        ));

        self.split_view.connect_sidebar_position_notify(clone!(
            #[weak(rename_to=appwindow)]
            obj,
            move |split_view| {
                update_widgets(split_view, &appwindow);
            }
        ));

        self.split_view.connect_collapsed_notify(clone!(
            #[strong]
            sidebar_expanded_shown,
            #[weak(rename_to=appwindow)]
            obj,
            move |split_view| {
                if split_view.is_collapsed() {
                    // Always hide sidebar when transitioning from non-collapsed to collapsed.
                    split_view.set_show_sidebar(false);
                } else {
                    // show sidebar again when it was shown before it was collapsed
                    if sidebar_expanded_shown.get() {
                        split_view.set_show_sidebar(true);
                    }
                    // update the shown state for when the sidebar was toggled shown in the collapsed state
                    sidebar_expanded_shown.set(split_view.shows_sidebar());
                }
                update_widgets(split_view, &appwindow);
            }
        ));
    }

    fn handle_righthanded_property(&self, righthanded: bool) {
        let obj = self.obj();

        if righthanded {
            obj.split_view().set_sidebar_position(PackType::Start);
            obj.main_header()
                .left_sidebar_reveal_toggle()
                .set_visible(true);
            obj.main_header()
                .right_sidebar_reveal_toggle()
                .set_visible(false);

            // obj.overlays().sidebar_box().set_halign(Align::Start);
            // obj.overlays()
            //     .sidebar_scroller()
            //     .set_placement(CornerType::TopRight);
        } else {
            obj.split_view().set_sidebar_position(PackType::End);
            obj.main_header()
                .left_sidebar_reveal_toggle()
                .set_visible(false);
            obj.main_header()
                .right_sidebar_reveal_toggle()
                .set_visible(true);

            // obj.overlays().sidebar_box().set_halign(Align::End);
            // obj.overlays()
            //     .sidebar_scroller()
            //     .set_placement(CornerType::TopLeft);
        }
    }
}
