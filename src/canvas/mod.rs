// Imports
use crate::{config, RnAppWindow};
use gtk::{
    gdk, gio, glib, glib::clone, prelude::*, subclass::prelude::*, Adjustment,
    DropTarget, EventControllerKey, EventControllerLegacy, IMMulticontext,
    PropagationPhase, Scrollable, ScrollablePolicy, Widget,
};
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};

#[derive(Debug, Default)]
struct Connections {
    tab_page_unsaved_changes: Option<glib::Binding>,
    tab_page_invalidate_thumbnail: Option<glib::SignalHandlerId>,
    appwindow_scalefactor: Option<glib::SignalHandlerId>,
    appwindow_save_in_progress: Option<glib::SignalHandlerId>,
    appwindow_unsaved_changes: Option<glib::SignalHandlerId>,
}

mod imp {

    use super::*;

    #[derive(Debug)]
    pub(crate) struct RnCanvas {
        pub(super) connections: RefCell<Connections>,

        // pub(crate) engine: RefCell<Engine>,
        pub(crate) engine_task_handler_handle: RefCell<Option<glib::JoinHandle<()>>>,

        pub(crate) save_in_progress: Cell<bool>,
        pub(crate) unsaved_changes: Cell<bool>,
        pub(crate) empty: Cell<bool>,

        pub(crate) last_export_dir: RefCell<Option<gio::File>>,
    }

    impl Default for RnCanvas {
        fn default() -> Self {
            let pointer_controller = EventControllerLegacy::builder()
                .name("pointer_controller")
                .propagation_phase(PropagationPhase::Bubble)
                .build();

            let key_controller = EventControllerKey::builder()
                .name("key_controller")
                .propagation_phase(PropagationPhase::Capture)
                .build();

            let key_controller_im_context = IMMulticontext::new();

            let invisible_cursor = gdk::Cursor::from_texture(
                &gdk::Texture::from_resource(
                    (String::from(config::APP_IDPATH)
                        + "icons/scalable/actions/cursor-invisible.svg")
                        .as_str(),
                ),
                32,
                32,
                gdk::Cursor::from_name("default", None).as_ref(),
            );

            // let engine = Engine::default();

            Self {
                connections: RefCell::new(Connections::default()),

                // engine: RefCell::new(Object),
                engine_task_handler_handle: RefCell::new(None),
                // is automatically updated whenever the output file changes.
                save_in_progress: Cell::new(false),
                unsaved_changes: Cell::new(false),
                empty: Cell::new(true),

                last_export_dir: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnCanvas {
        const NAME: &'static str = "RnCanvas";
        type Type = super::RnCanvas;
        type ParentType = Widget;
        type Interfaces = (Scrollable,);

        fn class_init(klass: &mut Self::Class) {
            // klass.set_layout_manager_type::<RnCanvasLayout>();
        }

        fn new() -> Self {
            Self::default()
        }
    }

    impl ObjectImpl for RnCanvas {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            self.setup_input();
        }

        fn dispose(&self) {
            self.obj().disconnect_connections();
            self.obj().abort_engine_task_handler();

            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    // this is nullable, so it can be used to represent Option<gio::File>
                    glib::ParamSpecBoolean::builder("save-in-progress")
                        .default_value(false)
                        .build(),
                    glib::ParamSpecBoolean::builder("unsaved-changes")
                        .default_value(false)
                        .build(),
                    glib::ParamSpecBoolean::builder("empty")
                        .default_value(true)
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "save-in-progress" => self.save_in_progress.get().to_value(),
                "unsaved-changes" => self.unsaved_changes.get().to_value(),
                "empty" => self.empty.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            let obj = self.obj();

            match pspec.name() {
                "save-in-progress" => {
                    let save_in_progress: bool =
                        value.get().expect("The value needs to be of type `bool`");
                    self.save_in_progress.replace(save_in_progress);
                }
                "unsaved-changes" => {
                    let unsaved_changes: bool =
                        value.get().expect("The value needs to be of type `bool`");
                    self.unsaved_changes.replace(unsaved_changes);
                }
                "empty" => {
                    let empty: bool =
                        value.get().expect("The value needs to be of type `bool`");
                    self.empty.replace(empty);
                    if empty {
                        obj.set_unsaved_changes(false);
                    }
                }
                _ => unimplemented!(),
            }
        }

        // fn signals() -> &'static [glib::subclass::Signal] {
        //     static SIGNALS: Lazy<Vec<glib::subclass::Signal>> = Lazy::new(|| {
        //         vec![
        //             glib::subclass::Signal::builder("handle-widget-flags")
        //                 .param_types([WidgetFlagsBoxed::static_type()])
        //                 .build(),
        //             glib::subclass::Signal::builder("invalidate-thumbnail").build(),
        //         ]
        //     });
        //     SIGNALS.as_ref()
        // }
    }

    impl WidgetImpl for RnCanvas {
        // request_mode(), measure(), allocate() overrides happen in the CanvasLayout LayoutManager

        fn snapshot(&self, snapshot: &gtk::Snapshot) {}
    }

    impl ScrollableImpl for RnCanvas {}

    impl RnCanvas {
        fn setup_input(&self) {
            let obj = self.obj();
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnCanvas(ObjectSubclass<imp::RnCanvas>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Scrollable;
}

impl Default for RnCanvas {
    fn default() -> Self {
        Self::new()
    }
}

// pub(crate) static OUTPUT_FILE_NEW_TITLE: once_cell::sync::Lazy<String> =
//     once_cell::sync::Lazy::new(|| "New Document");
// pub(crate) static OUTPUT_FILE_NEW_SUBTITLE: once_cell::sync::Lazy<String> =
//     once_cell::sync::Lazy::new(|| "Draft");

impl RnCanvas {
    // Sets the canvas zoom scroll step in % for one unit of the event controller delta
    pub(crate) const ZOOM_SCROLL_STEP: f64 = 0.1;

    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn unsaved_changes(&self) -> bool {
        self.property::<bool>("unsaved-changes")
    }

    #[allow(unused)]
    pub(crate) fn set_unsaved_changes(&self, unsaved_changes: bool) {
        if self.imp().unsaved_changes.get() != unsaved_changes {
            self.set_property("unsaved-changes", unsaved_changes.to_value());
        }
    }

    #[allow(unused)]
    pub(crate) fn empty(&self) -> bool {
        self.property::<bool>("empty")
    }

    #[allow(unused)]
    pub(crate) fn set_empty(&self, empty: bool) {
        if self.imp().empty.get() != empty {
            self.set_property("empty", empty.to_value());
        }
    }

    pub(crate) fn widget_size(&self) -> na::Vector2<f64> {
        na::vector![self.width() as f64, self.height() as f64]
    }

    /// Abort the engine task handler.
    ///
    /// Because the installed engine task handler holds a reference to the canvas,
    /// this MUST be called when the widget is removed from the widget tree,
    /// it's instance should be destroyed and it's memory should be freed.
    pub(crate) fn abort_engine_task_handler(&self) {
        if let Some(h) = self.imp().engine_task_handler_handle.take() {
            h.abort();
        }
    }

    pub(crate) fn save_engine_config(
        &self,
        settings: &gio::Settings,
    ) -> anyhow::Result<()> {
        // let engine_config = self.engine_ref().export_engine_config_as_json()?;
        // Ok(settings.set_string("engine-config", engine_config.as_str())?)
        Ok(())
    }

    pub(crate) fn load_engine_config_from_settings(
        &self,
        settings: &gio::Settings,
    ) -> anyhow::Result<()> {
        // load engine config
        let engine_config = settings.string("engine-config");
        Ok(())
    }

    /// Disconnect all connections with references to external objects
    /// to prepare moving the widget to another appwindow or closing it,
    /// when it is inside a tab page.
    pub(crate) fn disconnect_connections(&self) {
        let mut connections = self.imp().connections.borrow_mut();
        if let Some(old) = connections.appwindow_scalefactor.take() {
            self.disconnect(old);
        }
        if let Some(old) = connections.appwindow_save_in_progress.take() {
            self.disconnect(old);
        }
        if let Some(old) = connections.appwindow_unsaved_changes.take() {
            self.disconnect(old);
        }

        // tab page connections
        if let Some(old) = connections.tab_page_unsaved_changes.take() {
            old.unbind();
        }
        if let Some(old) = connections.tab_page_invalidate_thumbnail.take() {
            self.disconnect(old);
        }
    }

    /// When the widget is the child of a tab page, we want to connect their titles, icons, ..
    ///
    /// disconnects existing connections to old tab pages.
    pub(crate) fn connect_to_tab_page(&self, page: &adw::TabPage) {
        // update the tab title whenever the canvas output file changes
        // display unsaved changes as icon
        let tab_page_unsaved_changes = self
            .bind_property("unsaved-changes", page, "icon")
            .transform_to(|_, from: bool| {
                Some(from.then_some(gio::ThemedIcon::new("dot-symbolic")))
            })
            .sync_create()
            .build();

        // handle invalidating cached thumbnail in the tabs overview panel
        let tab_page_invalidate_thumbnail = self.connect_local(
            "invalidate-thumbnail",
            false,
            clone!(
                #[weak]
                page,
                #[upgrade_or]
                None,
                move |_| {
                    page.invalidate_thumbnail();
                    None
                }
            ),
        );

        let mut connections = self.imp().connections.borrow_mut();
        if let Some(old) = connections
            .tab_page_unsaved_changes
            .replace(tab_page_unsaved_changes)
        {
            old.unbind();
        }
        if let Some(old) = connections
            .tab_page_invalidate_thumbnail
            .replace(tab_page_invalidate_thumbnail)
        {
            self.disconnect(old);
        }
    }
}
