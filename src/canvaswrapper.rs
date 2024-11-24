// Imports
use crate::RnAppWindow;
use gtk::{
    gdk, glib, glib::clone, prelude::*, subclass::prelude::*, CompositeTemplate,
    CornerType, Entry, EventControllerMotion, EventControllerScroll,
    EventControllerScrollFlags, GestureDrag, GestureLongPress, GestureZoom, ListBox,
    PropagationPhase, ScrolledWindow, Widget,
};
use once_cell::sync::Lazy;
use std::cell::{Cell, RefCell};

#[derive(Debug, Default)]
struct Connections {
    appwindow_block_pinch_zoom_bind: Option<glib::Binding>,
    appwindow_show_scrollbars_bind: Option<glib::Binding>,
    appwindow_inertial_scrolling_bind: Option<glib::Binding>,
    appwindow_righthanded_bind: Option<glib::Binding>,
}

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/com/github/linruohan/mytool/ui/canvaswrapper.ui")]
    pub(crate) struct RnCanvasWrapper {
        pub(super) connections: RefCell<Connections>,
        pub(crate) canvas_touch_drawing_handler: RefCell<Option<glib::SignalHandlerId>>,
        pub(crate) show_scrollbars: Cell<bool>,
        pub(crate) block_pinch_zoom: Cell<bool>,
        pub(crate) inertial_scrolling: Cell<bool>,
        pub(crate) pointer_pos: Cell<Option<na::Vector2<f64>>>,
        pub(crate) last_contextmenu_pos: Cell<Option<na::Vector2<f64>>>,

        pub(crate) pointer_motion_controller: EventControllerMotion,
        pub(crate) canvas_drag_gesture: GestureDrag,
        pub(crate) canvas_zoom_gesture: GestureZoom,
        pub(crate) canvas_zoom_scroll_controller: EventControllerScroll,
        pub(crate) canvas_mouse_drag_middle_gesture: GestureDrag,
        pub(crate) canvas_alt_drag_gesture: GestureDrag,
        pub(crate) canvas_alt_shift_drag_gesture: GestureDrag,
        pub(crate) touch_two_finger_long_press_gesture: GestureLongPress,
        pub(crate) touch_long_press_gesture: GestureLongPress,

        #[template_child]
        pub(crate) scroller: TemplateChild<ScrolledWindow>,
        // #[template_child]
        // pub(crate) canvas: TemplateChild<RnCanvas>,
        // #[template_child]
        // pub(crate) contextmenu: TemplateChild<RnContextMenu>,
        #[template_child]
        pub entry: TemplateChild<Entry>,
        #[template_child]
        pub tasks_list: TemplateChild<ListBox>,
    }

    impl Default for RnCanvasWrapper {
        fn default() -> Self {
            let pointer_motion_controller = EventControllerMotion::builder()
                .name("pointer_motion_controller")
                .propagation_phase(PropagationPhase::Capture)
                .build();

            // This allows touch dragging and dragging with pointer in the empty space around the canvas.
            // All relevant pointer events for drawing are captured and denied for propagation before they arrive at this gesture.
            let canvas_drag_gesture = GestureDrag::builder()
                .name("canvas_drag_gesture")
                .button(gdk::BUTTON_PRIMARY)
                .exclusive(true)
                .propagation_phase(PropagationPhase::Bubble)
                .build();

            let canvas_zoom_gesture = GestureZoom::builder()
                .name("canvas_zoom_gesture")
                .propagation_phase(PropagationPhase::Capture)
                .build();

            let canvas_zoom_scroll_controller = EventControllerScroll::builder()
                .name("canvas_zoom_scroll_controller")
                .propagation_phase(PropagationPhase::Bubble)
                .flags(EventControllerScrollFlags::VERTICAL)
                .build();

            let canvas_mouse_drag_middle_gesture = GestureDrag::builder()
                .name("canvas_mouse_drag_middle_gesture")
                .button(gdk::BUTTON_MIDDLE)
                .exclusive(true)
                .propagation_phase(PropagationPhase::Bubble)
                .build();

            // alt + drag for panning with pointer
            let canvas_alt_drag_gesture = GestureDrag::builder()
                .name("canvas_alt_drag_gesture")
                .button(gdk::BUTTON_PRIMARY)
                .exclusive(true)
                .propagation_phase(PropagationPhase::Capture)
                .build();

            // alt + shift + drag for zooming with pointer
            let canvas_alt_shift_drag_gesture = GestureDrag::builder()
                .name("canvas_alt_shift_drag_gesture")
                .button(gdk::BUTTON_PRIMARY)
                .exclusive(true)
                .propagation_phase(PropagationPhase::Capture)
                .build();

            let touch_two_finger_long_press_gesture = GestureLongPress::builder()
                .name("touch_two_finger_long_press_gesture")
                .touch_only(true)
                .n_points(2)
                // activate a bit quicker
                .delay_factor(0.8)
                .propagation_phase(PropagationPhase::Capture)
                .build();

            let touch_long_press_gesture = GestureLongPress::builder()
                .name("touch_long_press_gesture")
                .touch_only(true)
                .build();

            Self {
                connections: RefCell::new(Connections::default()),
                canvas_touch_drawing_handler: RefCell::new(None),
                show_scrollbars: Cell::new(false),
                block_pinch_zoom: Cell::new(false),
                inertial_scrolling: Cell::new(true),
                pointer_pos: Cell::new(None),
                last_contextmenu_pos: Cell::new(None),

                pointer_motion_controller,
                canvas_drag_gesture,
                canvas_zoom_gesture,
                canvas_zoom_scroll_controller,
                canvas_mouse_drag_middle_gesture,
                canvas_alt_drag_gesture,
                canvas_alt_shift_drag_gesture,
                touch_two_finger_long_press_gesture,
                touch_long_press_gesture,

                scroller: TemplateChild::<ScrolledWindow>::default(),
                // canvas: TemplateChild::<RnCanvas>::default(),
                // contextmenu: TemplateChild::<RnContextMenu>::default(),
                entry: TemplateChild::<Entry>::default(),
                tasks_list: TemplateChild::<ListBox>::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnCanvasWrapper {
        const NAME: &'static str = "RnCanvasWrapper";
        type Type = super::RnCanvasWrapper;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnCanvasWrapper {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Add input controllers
            self.scroller
                .add_controller(self.pointer_motion_controller.clone());
            self.scroller
                .add_controller(self.canvas_drag_gesture.clone());
            self.scroller
                .add_controller(self.canvas_zoom_gesture.clone());
            self.scroller
                .add_controller(self.canvas_zoom_scroll_controller.clone());
            self.scroller
                .add_controller(self.canvas_mouse_drag_middle_gesture.clone());
            self.scroller
                .add_controller(self.canvas_alt_drag_gesture.clone());
            self.scroller
                .add_controller(self.canvas_alt_shift_drag_gesture.clone());
            self.scroller
                .add_controller(self.touch_two_finger_long_press_gesture.clone());
            // self.canvas
            // .add_controller(self.touch_long_press_gesture.clone());

            // group
            self.touch_two_finger_long_press_gesture
                .group_with(&self.canvas_zoom_gesture);

            self.setup_input();

            // let canvas_touch_drawing_handler = self.canvas.connect_notify_local(
            //     Some("touch-drawing"),
            //     clone!(
            //         #[weak(rename_to=canvaswrapper)]
            //         obj,
            //         move |_canvas, _pspec| {
            //             // Disable the zoom gesture and kinetic scrolling when touch drawing is enabled.
            //             canvaswrapper.imp().canvas_kinetic_scrolling_update();
            //             canvaswrapper.imp().canvas_zoom_gesture_update();
            //         }
            //     ),
            // );

            // self.canvas_touch_drawing_handler
            //     .replace(Some(canvas_touch_drawing_handler));
        }

        fn dispose(&self) {
            self.obj().disconnect_connections();

            if let Some(handler) = self.canvas_touch_drawing_handler.take() {
                // self.canvas.disconnect(handler);
            }

            // the engine task handler needs to be be aborted here,
            // else a reference of the canvas is held forever in the handler causing a memory leak.
            // self.canvas.abort_engine_task_handler();

            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }

        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecBoolean::builder("show-scrollbars")
                        .default_value(false)
                        .build(),
                    glib::ParamSpecBoolean::builder("block-pinch-zoom")
                        .default_value(false)
                        .build(),
                    glib::ParamSpecBoolean::builder("inertial-scrolling")
                        .default_value(true)
                        .build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "show-scrollbars" => self.show_scrollbars.get().to_value(),
                "block-pinch-zoom" => self.block_pinch_zoom.get().to_value(),
                "inertial-scrolling" => self.inertial_scrolling.get().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(
            &self,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "show-scrollbars" => {
                    let show_scrollbars = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`");
                    self.show_scrollbars.replace(show_scrollbars);

                    self.scroller.hscrollbar().set_visible(show_scrollbars);
                    self.scroller.vscrollbar().set_visible(show_scrollbars);
                }
                "block-pinch-zoom" => {
                    let block_pinch_zoom = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`");
                    self.block_pinch_zoom.replace(block_pinch_zoom);
                    self.canvas_zoom_gesture_update();
                }
                "inertial-scrolling" => {
                    let inertial_scrolling = value
                        .get::<bool>()
                        .expect("The value needs to be of type `bool`");

                    self.inertial_scrolling.replace(inertial_scrolling);
                    self.canvas_kinetic_scrolling_update();
                }
                _ => unimplemented!(),
            }
        }
    }

    impl WidgetImpl for RnCanvasWrapper {}

    impl RnCanvasWrapper {
        fn canvas_zoom_gesture_update(&self) {
            // if !self.block_pinch_zoom.get() && !self.canvas.touch_drawing() {
            //     self.canvas_zoom_gesture
            //         .set_propagation_phase(PropagationPhase::Capture);
            // } else {
            //     self.canvas_zoom_gesture
            //         .set_propagation_phase(PropagationPhase::None);
            // }
        }

        fn canvas_kinetic_scrolling_update(&self) {
            // self.scroller.set_kinetic_scrolling(
            //     !self.canvas.touch_drawing() && self.inertial_scrolling.get(),
            // );
        }

        fn setup_input(&self) {
            let obj = self.obj();

            {
                self.pointer_motion_controller.connect_motion(clone!(
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_, x, y| {
                        canvaswrapper.imp().pointer_pos.set(Some(na::vector![x, y]));
                    }
                ));

                self.pointer_motion_controller.connect_leave(clone!(
                    #[weak(rename_to=canvaswrapper)]
                    obj,
                    move |_| {
                        canvaswrapper.imp().pointer_pos.set(None);
                    }
                ));
            }

            // Actions when moving view with controls provided by the scroller ScrolledWindow.
        }
    }
}

glib::wrapper! {
    pub(crate) struct RnCanvasWrapper(ObjectSubclass<imp::RnCanvasWrapper>)
    @extends Widget;
}

impl Default for RnCanvasWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl RnCanvasWrapper {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    #[allow(unused)]
    pub(crate) fn show_scrollbars(&self) -> bool {
        self.property::<bool>("show-scrollbars")
    }

    #[allow(unused)]
    pub(crate) fn set_show_scrollbars(&self, show_scrollbars: bool) {
        self.set_property("show-scrollbars", show_scrollbars.to_value());
    }
    #[allow(unused)]
    pub(crate) fn block_pinch_zoom(&self) -> bool {
        self.property::<bool>("block-pinch-zoom")
    }

    #[allow(unused)]
    pub(crate) fn set_block_pinch_zoom(&self, block_pinch_zoom: bool) {
        self.set_property("block-pinch-zoom", block_pinch_zoom);
    }

    #[allow(unused)]
    pub(crate) fn inertial_scrolling(&self) -> bool {
        self.property::<bool>("inertial-scrolling")
    }

    #[allow(unused)]
    pub(crate) fn set_inertial_scrolling(&self, inertial_scrolling: bool) {
        self.set_property("inertial-scrolling", inertial_scrolling);
    }
    #[allow(unused)]
    pub(crate) fn last_contextmenu_pos(&self) -> Option<na::Vector2<f64>> {
        self.imp().last_contextmenu_pos.get()
    }
    #[allow(unused)]
    pub(crate) fn scroller(&self) -> ScrolledWindow {
        self.imp().scroller.get()
    }

    // pub(crate) fn canvas(&self) -> RnCanvas {
    //     self.imp().canvas.get()
    // }
    // #[allow(unused)]
    // pub(crate) fn contextmenu(&self) -> RnContextMenu {
    //     self.imp().contextmenu.get()
    // }
    #[allow(unused)]
    /// Initializes for the given appwindow. Usually `init()` is only called once,
    /// but because this widget can be moved across appwindows through tabs,
    /// this function also disconnects and replaces all existing old connections
    ///
    /// The same method of the canvas child is chained up in here.
    pub(crate) fn init_reconnect(&self, appwindow: &RnAppWindow) {
        // self.imp().canvas.init_reconnect(appwindow);

        let appwindow_block_pinch_zoom_bind = appwindow
            .bind_property("block-pinch-zoom", self, "block_pinch_zoom")
            .sync_create()
            .build();

        let appwindow_righthanded_bind = appwindow
            .bind_property("righthanded", &self.scroller(), "window-placement")
            .transform_to(|_, righthanded: bool| {
                if righthanded {
                    Some(CornerType::BottomRight)
                } else {
                    Some(CornerType::BottomLeft)
                }
            })
            .sync_create()
            .build();

        let mut connections = self.imp().connections.borrow_mut();
        if let Some(old) = connections
            .appwindow_block_pinch_zoom_bind
            .replace(appwindow_block_pinch_zoom_bind)
        {
            old.unbind()
        }

        if let Some(old) = connections
            .appwindow_righthanded_bind
            .replace(appwindow_righthanded_bind)
        {
            old.unbind();
        }
    }

    /// This disconnects all connections with references to external objects,
    /// to prepare moving the widget to another appwindow.
    ///
    /// The same method of the canvas child is chained up in here.
    pub(crate) fn disconnect_connections(&self) {
        // self.canvas().disconnect_connections();

        let mut connections = self.imp().connections.borrow_mut();
        if let Some(old) = connections.appwindow_block_pinch_zoom_bind.take() {
            old.unbind();
        }
        if let Some(old) = connections.appwindow_show_scrollbars_bind.take() {
            old.unbind();
        }
        if let Some(old) = connections.appwindow_inertial_scrolling_bind.take() {
            old.unbind();
        }
        if let Some(old) = connections.appwindow_righthanded_bind.take() {
            old.unbind();
        }
    }
    #[allow(unused)]
    /// When the widget is the child of a tab page, we want to connect the title, icons, ..
    ///
    /// disconnects existing connections to old tab pages.
    ///
    /// The same method of the canvas child is chained up in here.
    pub(crate) fn connect_to_tab_page(&self, page: &adw::TabPage) {
        // self.canvas().connect_to_tab_page(page);
    }
}
