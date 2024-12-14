// Imports
use crate::{config, RnMainHeader, RnSidebar};
use adw::{prelude::*, subclass::prelude::*, OverlaySplitView, ViewStack};
use gtk::{
    gdk, glib, glib::clone, CompositeTemplate, CssProvider, FilterListModel, PackType,
};
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
#[derive(Debug, CompositeTemplate)]
#[template(resource = "/com/github/linruohan/mytool/ui/appwindow.ui")]
pub(crate) struct RnAppWindow {
    pub(crate) current_filter_model: RefCell<Option<FilterListModel>>,
    pub(crate) righthanded: Cell<bool>,

    #[template_child]
    pub(crate) view_stack: TemplateChild<ViewStack>,
    #[template_child]
    pub(crate) main_header: TemplateChild<RnMainHeader>,
    #[template_child]
    pub(crate) overlay_split_view: TemplateChild<OverlaySplitView>,
    #[template_child]
    pub(crate) views_split_view: TemplateChild<OverlaySplitView>,
    #[template_child]
    pub(crate) sidebar: TemplateChild<RnSidebar>,
    #[template_child]
    pub(crate) views_stack: TemplateChild<ViewStack>,
}

impl Default for RnAppWindow {
    fn default() -> Self {
        Self {
            current_filter_model: RefCell::new(None),
            righthanded: Cell::new(true),

            view_stack: TemplateChild::<ViewStack>::default(),
            main_header: TemplateChild::<RnMainHeader>::default(),
            overlay_split_view: TemplateChild::<adw::OverlaySplitView>::default(),
            views_split_view: TemplateChild::<adw::OverlaySplitView>::default(),
            sidebar: TemplateChild::<RnSidebar>::default(),
            views_stack: TemplateChild::<ViewStack>::default(),
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
        klass.install_action_async("win.todo-view", None, |window, _, _| async move {
            window.views_stack().set_visible_child_name("done_page");
        });
        klass.install_action_async("win.work-view", None, |window, _, _| async move {
            window
                .views_stack()
                .set_visible_child_name("workspacebrowser_page");
        });
        klass.install_action_async(
            "win.reminder-view",
            None,
            |window, _, _| async move {
                window.views_stack().set_visible_child_name("reminder_page");
            },
        );
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

        self.setup_split_view();
    }

    fn dispose(&self) {
        self.dispose_template();
        while let Some(child) = self.obj().first_child() {
            child.unparent();
        }
    }

    fn properties() -> &'static [glib::ParamSpec] {
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![glib::ParamSpecBoolean::builder("righthanded")
                .default_value(false)
                .build()]
        });
        PROPERTIES.as_ref()
    }

    fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "righthanded" => self.righthanded.get().to_value(),

            _ => unimplemented!(),
        }
    }

    fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
        match pspec.name() {
            "righthanded" => {
                let righthanded = value
                    .get::<bool>()
                    .expect("The value needs to be of type `bool`");

                self.righthanded.replace(righthanded);

                self.handle_righthanded_property(righthanded);
            }

            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for RnAppWindow {}

impl WindowImpl for RnAppWindow {
    fn close_request(&self) -> glib::Propagation {
        self.main_header.headerbar().set_sensitive(false);
        self.sidebar.headerbar().set_sensitive(false);

        // 保存 collections
        // if let Some(wrapper) = self.obj().active_tab_wrapper() {
        //     wrapper.save_collections();
        // }

        // Inhibit (Overwrite) the default handler. This handler is then responsible for destroying the window.
        glib::Propagation::Stop
    }
}

impl ApplicationWindowImpl for RnAppWindow {}
impl AdwWindowImpl for RnAppWindow {}
impl AdwApplicationWindowImpl for RnAppWindow {}

impl RnAppWindow {
    fn setup_split_view(&self) {
        let obj = self.obj();
        let overlay_split_view = self.overlay_split_view.get();
        let left_sidebar_reveal_toggle = obj.main_header().left_sidebar_reveal_toggle();
        let right_sidebar_reveal_toggle =
            obj.main_header().right_sidebar_reveal_toggle();

        left_sidebar_reveal_toggle
            .bind_property("active", &right_sidebar_reveal_toggle, "active")
            .sync_create()
            .bidirectional()
            .build();

        left_sidebar_reveal_toggle
            .bind_property("active", &overlay_split_view, "show-sidebar")
            .sync_create()
            .bidirectional()
            .build();
        right_sidebar_reveal_toggle
            .bind_property("active", &overlay_split_view, "show-sidebar")
            .sync_create()
            .bidirectional()
            .build();

        let update_widgets =
            move |overlay_split_view: &adw::OverlaySplitView,
                  appwindow: &super::RnAppWindow| {
                let sidebar_position = overlay_split_view.sidebar_position();
                let sidebar_collapsed = overlay_split_view.is_collapsed();
                let sidebar_shown = overlay_split_view.shows_sidebar();

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

        self.overlay_split_view.connect_show_sidebar_notify(clone!(
            #[strong]
            sidebar_expanded_shown,
            #[weak(rename_to=appwindow)]
            obj,
            move |overlay_split_view| {
                if !overlay_split_view.is_collapsed() {
                    sidebar_expanded_shown.set(overlay_split_view.shows_sidebar());
                }
                update_widgets(overlay_split_view, &appwindow);
            }
        ));

        self.overlay_split_view
            .connect_sidebar_position_notify(clone!(
                #[weak(rename_to=appwindow)]
                obj,
                move |overlay_split_view| {
                    update_widgets(overlay_split_view, &appwindow);
                }
            ));

        self.overlay_split_view.connect_collapsed_notify(clone!(
            #[strong]
            sidebar_expanded_shown,
            #[weak(rename_to=appwindow)]
            obj,
            move |overlay_split_view| {
                if overlay_split_view.is_collapsed() {
                    // Always hide sidebar when transitioning from non-collapsed to collapsed.
                    overlay_split_view.set_show_sidebar(false);
                } else {
                    // show sidebar again when it was shown before it was collapsed
                    if sidebar_expanded_shown.get() {
                        overlay_split_view.set_show_sidebar(true);
                    }
                    // update the shown state for when the sidebar was toggled shown in the collapsed state
                    sidebar_expanded_shown.set(overlay_split_view.shows_sidebar());
                }
                update_widgets(overlay_split_view, &appwindow);
            }
        ));
    }

    fn handle_righthanded_property(&self, righthanded: bool) {
        let obj = self.obj();

        if righthanded {
            obj.overlay_split_view()
                .set_sidebar_position(PackType::Start);
            obj.main_header()
                .left_sidebar_reveal_toggle()
                .set_visible(true);
            obj.main_header()
                .right_sidebar_reveal_toggle()
                .set_visible(false);
        } else {
            obj.overlay_split_view().set_sidebar_position(PackType::End);
            obj.main_header()
                .left_sidebar_reveal_toggle()
                .set_visible(false);
            obj.main_header()
                .right_sidebar_reveal_toggle()
                .set_visible(true);
        }
    }
}
