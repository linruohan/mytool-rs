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
    hadjustment: Option<glib::SignalHandlerId>,
    vadjustment: Option<glib::SignalHandlerId>,
    tab_page_output_file: Option<glib::Binding>,
    tab_page_unsaved_changes: Option<glib::Binding>,
    tab_page_invalidate_thumbnail: Option<glib::SignalHandlerId>,
    appwindow_output_file: Option<glib::SignalHandlerId>,
    appwindow_scalefactor: Option<glib::SignalHandlerId>,
    appwindow_save_in_progress: Option<glib::SignalHandlerId>,
    appwindow_unsaved_changes: Option<glib::SignalHandlerId>,
    appwindow_touch_drawing: Option<glib::Binding>,
    appwindow_show_drawing_cursor: Option<glib::Binding>,
    appwindow_regular_cursor: Option<glib::Binding>,
    appwindow_drawing_cursor: Option<glib::Binding>,
    appwindow_drop_target: Option<glib::SignalHandlerId>,
    appwindow_handle_widget_flags: Option<glib::SignalHandlerId>,
}

mod imp {

    use super::*;

    #[derive(Debug)]
    pub(crate) struct RnCanvas {
        pub(super) connections: RefCell<Connections>,
        pub(crate) hadjustment: RefCell<Option<Adjustment>>,
        pub(crate) vadjustment: RefCell<Option<Adjustment>>,
        pub(crate) hscroll_policy: Cell<ScrollablePolicy>,
        pub(crate) vscroll_policy: Cell<ScrollablePolicy>,
        pub(crate) regular_cursor_icon_name: RefCell<String>,
        pub(crate) regular_cursor: RefCell<gdk::Cursor>,
        pub(crate) drawing_cursor_icon_name: RefCell<String>,
        pub(crate) drawing_cursor: RefCell<gdk::Cursor>,
        pub(crate) invisible_cursor: RefCell<gdk::Cursor>,
        pub(crate) pointer_controller: EventControllerLegacy,
        pub(crate) key_controller: EventControllerKey,
        pub(crate) key_controller_im_context: IMMulticontext,
        pub(crate) drop_target: DropTarget,
        pub(crate) drawing_cursor_enabled: Cell<bool>,

        // pub(crate) engine: RefCell<Engine>,
        pub(crate) engine_task_handler_handle: RefCell<Option<glib::JoinHandle<()>>>,

        pub(crate) output_file: RefCell<Option<gio::File>>,
        pub(crate) output_file_watcher_task: RefCell<Option<glib::JoinHandle<()>>>,
        pub(crate) output_file_modified_toast_singleton: glib::WeakRef<adw::Toast>,
        pub(crate) output_file_expect_write: Cell<bool>,
        pub(crate) save_in_progress: Cell<bool>,
        pub(crate) unsaved_changes: Cell<bool>,
        pub(crate) empty: Cell<bool>,
        pub(crate) touch_drawing: Cell<bool>,
        pub(crate) show_drawing_cursor: Cell<bool>,

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

            let drop_target = DropTarget::builder()
                .name("canvas_drop_target")
                .propagation_phase(PropagationPhase::Capture)
                .actions(gdk::DragAction::COPY)
                .build();

            // the order here is important: first files, then text
            drop_target
                .set_types(&[gio::File::static_type(), glib::types::Type::STRING]);

            let regular_cursor_icon_name = String::from("cursor-dot-medium");
            let regular_cursor = gdk::Cursor::from_texture(
                &gdk::Texture::from_resource(
                    (String::from(config::APP_IDPATH)
                        + "icons/scalable/actions/cursor-dot-medium.svg")
                        .as_str(),
                ),
                32,
                32,
                gdk::Cursor::from_name("default", None).as_ref(),
            );
            let drawing_cursor_icon_name = String::from("cursor-dot-small");
            let drawing_cursor = gdk::Cursor::from_texture(
                &gdk::Texture::from_resource(
                    (String::from(config::APP_IDPATH)
                        + "icons/scalable/actions/cursor-dot-small.svg")
                        .as_str(),
                ),
                32,
                32,
                gdk::Cursor::from_name("default", None).as_ref(),
            );

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

                hadjustment: RefCell::new(None),
                vadjustment: RefCell::new(None),
                hscroll_policy: Cell::new(ScrollablePolicy::Minimum),
                vscroll_policy: Cell::new(ScrollablePolicy::Minimum),
                regular_cursor: RefCell::new(regular_cursor),
                regular_cursor_icon_name: RefCell::new(regular_cursor_icon_name),
                drawing_cursor: RefCell::new(drawing_cursor),
                drawing_cursor_icon_name: RefCell::new(drawing_cursor_icon_name),
                invisible_cursor: RefCell::new(invisible_cursor),
                pointer_controller,
                key_controller,
                key_controller_im_context,
                drop_target,
                drawing_cursor_enabled: Cell::new(false),

                // engine: RefCell::new(Object),
                engine_task_handler_handle: RefCell::new(None),

                output_file: RefCell::new(None),
                output_file_watcher_task: RefCell::new(None),
                // is automatically updated whenever the output file changes.
                output_file_modified_toast_singleton: glib::WeakRef::new(),
                output_file_expect_write: Cell::new(false),
                save_in_progress: Cell::new(false),
                unsaved_changes: Cell::new(false),
                empty: Cell::new(true),
                touch_drawing: Cell::new(false),
                show_drawing_cursor: Cell::new(false),

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

            // while let Some(child) = self.obj().first_child() {
            //     child.unparent();
            // }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    // this is nullable, so it can be used to represent Option<gio::File>
                    glib::ParamSpecObject::builder::<gio::File>("output-file").build(),
                    glib::ParamSpecBoolean::builder("save-in-progress")
                        .default_value(false)
                        .build(),
                    glib::ParamSpecBoolean::builder("unsaved-changes")
                        .default_value(false)
                        .build(),
                    glib::ParamSpecBoolean::builder("empty")
                        .default_value(true)
                        .build(),
                    glib::ParamSpecBoolean::builder("touch-drawing")
                        .default_value(false)
                        .build(),
                    glib::ParamSpecBoolean::builder("show-drawing-cursor")
                        .default_value(true)
                        .build(),
                    glib::ParamSpecString::builder("regular-cursor")
                        .default_value(Some("cursor-dot-medium"))
                        .build(),
                    glib::ParamSpecString::builder("drawing-cursor")
                        .default_value(Some("cursor-dot-small"))
                        .build(),
                    // Scrollable properties
                    glib::ParamSpecOverride::for_interface::<Scrollable>(
                        "hscroll-policy",
                    ),
                    glib::ParamSpecOverride::for_interface::<Scrollable>(
                        "vscroll-policy",
                    ),
                    glib::ParamSpecOverride::for_interface::<Scrollable>("hadjustment"),
                    glib::ParamSpecOverride::for_interface::<Scrollable>("vadjustment"),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "output-file" => self.output_file.borrow().to_value(),
                "save-in-progress" => self.save_in_progress.get().to_value(),
                "unsaved-changes" => self.unsaved_changes.get().to_value(),
                "empty" => self.empty.get().to_value(),
                "hadjustment" => self.hadjustment.borrow().to_value(),
                "vadjustment" => self.vadjustment.borrow().to_value(),
                "hscroll-policy" => self.hscroll_policy.get().to_value(),
                "vscroll-policy" => self.vscroll_policy.get().to_value(),
                "touch-drawing" => self.touch_drawing.get().to_value(),
                "show-drawing-cursor" => self.show_drawing_cursor.get().to_value(),
                "regular-cursor" => self.regular_cursor_icon_name.borrow().to_value(),
                "drawing-cursor" => self.drawing_cursor_icon_name.borrow().to_value(),
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
                "output-file" => {
                    let output_file = value
                        .get::<Option<gio::File>>()
                        .expect("The value needs to be of type `Option<gio::File>`");
                    self.output_file.replace(output_file);
                }
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

                "hscroll-policy" => {
                    let hscroll_policy = value.get().unwrap();
                    self.hscroll_policy.replace(hscroll_policy);
                }

                "vscroll-policy" => {
                    let vscroll_policy = value.get().unwrap();
                    self.vscroll_policy.replace(vscroll_policy);
                }
                "touch-drawing" => {
                    let touch_drawing: bool =
                        value.get().expect("The value needs to be of type `bool`");
                    self.touch_drawing.replace(touch_drawing);
                }
                "show-drawing-cursor" => {
                    let show_drawing_cursor: bool =
                        value.get().expect("The value needs to be of type `bool`");
                    self.show_drawing_cursor.replace(show_drawing_cursor);

                    // if self.drawing_cursor_enabled.get() {
                    //     if show_drawing_cursor {
                    //         obj.set_cursor(Some(&*self.drawing_cursor.borrow()));
                    //     } else {
                    //         obj.set_cursor(Some(&*self.invisible_cursor.borrow()));
                    //     }
                    // } else {
                    //     obj.set_cursor(Some(&*self.regular_cursor.borrow()));
                    // }
                }
                "regular-cursor" => {
                    let icon_name = value.get().unwrap();
                    self.regular_cursor_icon_name.replace(icon_name);

                    let cursor = gdk::Cursor::from_texture(
                        &gdk::Texture::from_resource(
                            (String::from(config::APP_IDPATH)
                                + &format!(
                                    "icons/scalable/actions/{}.svg",
                                    self.regular_cursor_icon_name.borrow()
                                ))
                                .as_str(),
                        ),
                        32,
                        32,
                        gdk::Cursor::from_name("default", None).as_ref(),
                    );

                    self.regular_cursor.replace(cursor);

                    // obj.set_cursor(Some(&*self.regular_cursor.borrow()));
                }
                "drawing-cursor" => {
                    let icon_name = value.get().unwrap();
                    self.drawing_cursor_icon_name.replace(icon_name);

                    let cursor = gdk::Cursor::from_texture(
                        &gdk::Texture::from_resource(
                            (String::from(config::APP_IDPATH)
                                + &format!(
                                    "icons/scalable/actions/{}.svg",
                                    self.drawing_cursor_icon_name.borrow()
                                ))
                                .as_str(),
                        ),
                        32,
                        32,
                        gdk::Cursor::from_name("default", None).as_ref(),
                    );

                    self.drawing_cursor.replace(cursor);
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
    pub(crate) fn regular_cursor(&self) -> String {
        self.property::<String>("regular-cursor")
    }

    #[allow(unused)]
    pub(crate) fn set_regular_cursor(&self, regular_cursor: &str) {
        self.set_property("regular-cursor", regular_cursor.to_value());
    }

    #[allow(unused)]
    pub(crate) fn drawing_cursor(&self) -> String {
        self.property::<String>("drawing-cursor")
    }

    #[allow(unused)]
    pub(crate) fn set_drawing_cursor(&self, drawing_cursor: &str) {
        self.set_property("drawing-cursor", drawing_cursor.to_value());
    }

    #[allow(unused)]
    pub(crate) fn output_file(&self) -> Option<gio::File> {
        self.property::<Option<gio::File>>("output-file")
    }

    #[allow(unused)]
    pub(crate) fn set_output_file(&self, output_file: Option<gio::File>) {
        self.set_property("output-file", output_file.to_value());
    }

    #[allow(unused)]
    pub(crate) fn output_file_expect_write(&self) -> bool {
        self.imp().output_file_expect_write.get()
    }

    #[allow(unused)]
    pub(crate) fn set_output_file_expect_write(&self, expect_write: bool) {
        self.imp().output_file_expect_write.set(expect_write);
    }

    #[allow(unused)]
    pub(crate) fn save_in_progress(&self) -> bool {
        self.property::<bool>("save-in-progress")
    }

    #[allow(unused)]
    pub(crate) fn set_save_in_progress(&self, save_in_progress: bool) {
        if self.imp().save_in_progress.get() != save_in_progress {
            self.set_property("save-in-progress", save_in_progress.to_value());
        }
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

    #[allow(unused)]
    pub(crate) fn touch_drawing(&self) -> bool {
        self.property::<bool>("touch-drawing")
    }

    #[allow(unused)]
    pub(crate) fn set_touch_drawing(&self, touch_drawing: bool) {
        if self.imp().touch_drawing.get() != touch_drawing {
            self.set_property("touch-drawing", touch_drawing.to_value());
        }
    }

    #[allow(unused)]
    pub(crate) fn show_drawing_cursor(&self) -> bool {
        self.property::<bool>("show-drawing-cursor")
    }

    #[allow(unused)]
    pub(crate) fn set_show_drawing_cursor(&self, show_drawing_cursor: bool) {
        if self.imp().show_drawing_cursor.get() != show_drawing_cursor {
            self.set_property("show-drawing-cursor", show_drawing_cursor.to_value());
        }
    }

    pub(super) fn emit_invalidate_thumbnail(&self) {
        self.emit_by_name::<()>("invalidate-thumbnail", &[]);
    }

    pub(crate) fn last_export_dir(&self) -> Option<gio::File> {
        self.imp().last_export_dir.borrow().clone()
    }

    pub(crate) fn set_last_export_dir(&self, dir: Option<gio::File>) {
        self.imp().last_export_dir.replace(dir);
    }

    pub(crate) fn configure_adjustments(
        &self,
        widget_size: na::Vector2<f64>,
        offset_mins_maxs: (na::Vector2<f64>, na::Vector2<f64>),
        offset: na::Vector2<f64>,
    ) {
        let (offset_mins, offset_maxs) = offset_mins_maxs;

        if let Some(hadj) = self.hadjustment() {
            hadj.configure(
                // This gets clamped to the lower and upper values
                offset[0],
                offset_mins[0],
                offset_maxs[0],
                0.1 * widget_size[0],
                0.9 * widget_size[0],
                widget_size[0],
            )
        };

        if let Some(vadj) = self.vadjustment() {
            vadj.configure(
                // This gets clamped to the lower and upper values
                offset[1],
                offset_mins[1],
                offset_maxs[1],
                0.1 * widget_size[1],
                0.9 * widget_size[1],
                widget_size[1],
            );
        }

        self.queue_resize();
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

    pub(crate) fn set_text_preprocessing(&self, enable: bool) {
        if enable {
            self.imp()
                .key_controller
                .set_im_context(Some(&self.imp().key_controller_im_context));
        } else {
            self.imp()
                .key_controller
                .set_im_context(None::<&IMMulticontext>);
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

    /// Switches between the regular and the drawing cursor
    pub(crate) fn enable_drawing_cursor(&self, drawing_cursor: bool) {
        if drawing_cursor == self.imp().drawing_cursor_enabled.get() {
            return;
        };
        self.imp().drawing_cursor_enabled.set(drawing_cursor);

        if drawing_cursor {
            if self.imp().show_drawing_cursor.get() {
                self.set_cursor(Some(&*self.imp().drawing_cursor.borrow()));
            } else {
                self.set_cursor(Some(&*self.imp().invisible_cursor.borrow()));
            }
        } else {
            self.set_cursor(Some(&*self.imp().regular_cursor.borrow()));
        }
    }

    pub(crate) fn clear_output_file_watcher(&self) {
        if let Some(handle) = self.imp().output_file_watcher_task.take() {
            handle.abort();
        }
    }

    pub(crate) fn dismiss_output_file_modified_toast(&self) {
        if let Some(output_file_modified_toast) =
            self.imp().output_file_modified_toast_singleton.upgrade()
        {
            output_file_modified_toast.dismiss();
        }
    }

    /// Replaces and installs a new file monitor when there is an output file present
    fn reinstall_output_file_watcher(&self, appwindow: &RnAppWindow) {
        if let Some(output_file) = self.output_file() {
            // self.create_output_file_watcher(&output_file, appwindow);
        } else {
            self.clear_output_file_watcher();
        }
    }

    /// Disconnect all connections with references to external objects
    /// to prepare moving the widget to another appwindow or closing it,
    /// when it is inside a tab page.
    pub(crate) fn disconnect_connections(&self) {
        self.clear_output_file_watcher();

        let mut connections = self.imp().connections.borrow_mut();
        if let Some(old) = connections.appwindow_output_file.take() {
            self.disconnect(old);
        }
        if let Some(old) = connections.appwindow_scalefactor.take() {
            self.disconnect(old);
        }
        if let Some(old) = connections.appwindow_save_in_progress.take() {
            self.disconnect(old);
        }
        if let Some(old) = connections.appwindow_unsaved_changes.take() {
            self.disconnect(old);
        }
        if let Some(old) = connections.appwindow_touch_drawing.take() {
            old.unbind();
        }
        if let Some(old) = connections.appwindow_show_drawing_cursor.take() {
            old.unbind();
        }
        if let Some(old) = connections.appwindow_regular_cursor.take() {
            old.unbind();
        }
        if let Some(old) = connections.appwindow_drawing_cursor.take() {
            old.unbind();
        }
        if let Some(old) = connections.appwindow_drop_target.take() {
            self.imp().drop_target.disconnect(old);
        }
        if let Some(old) = connections.appwindow_handle_widget_flags.take() {
            self.disconnect(old);
        }

        // tab page connections
        if let Some(old) = connections.tab_page_output_file.take() {
            old.unbind();
        }
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
        // let tab_page_output_file = self
        //     .bind_property("output-file", page, "title")
        //     .sync_create()
        //     .transform_to(|b, _output_file: Option<gio::File>| {
        //         Some(
        //             b.source()?
        //                 .downcast::<RnCanvas>()
        //                 .unwrap()
        //                 .doc_title_display(),
        //         )
        //     })
        //     .build();

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
        // if let Some(old) = connections
        //     .tab_page_output_file
        //     .replace(tab_page_output_file)
        // {
        //     old.unbind();
        // }
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
