// Imports
use crate::{FilterPaneRow, FilterType, RnAppMenu, RnAppWindow};
use adw::{prelude::*, subclass::prelude::*};
use gtk::{glib, glib::clone, Button, CompositeTemplate, FlowBox, Widget};
mod imp {

    use super::*;

    #[derive(Debug, CompositeTemplate, Default)]
    #[template(resource = "/com/github/linruohan/mytool/ui/sidebar.ui")]
    pub(crate) struct RnSidebar {
        #[template_child]
        pub(crate) headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub(crate) left_close_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) right_close_button: TemplateChild<Button>,
        #[template_child]
        pub(crate) appmenu: TemplateChild<RnAppMenu>,
        #[template_child]
        pub(crate) sidebar_stack: TemplateChild<adw::ViewStack>,
        // #[template_child]
        // pub(crate) workspacebrowser: TemplateChild<RnWorkspaceBrowser>,
        // #[template_child]
        // pub(crate) settings_panel: TemplateChild<RnSettingsPanel>,
        #[template_child]
        pub(crate) filters_flow: TemplateChild<FlowBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnSidebar {
        const NAME: &'static str = "RnSidebar";
        type Type = super::RnSidebar;
        type ParentType = Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for RnSidebar {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            self.dispose_template();
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for RnSidebar {}
}

glib::wrapper! {
    pub(crate) struct RnSidebar(ObjectSubclass<imp::RnSidebar>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for RnSidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl RnSidebar {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn headerbar(&self) -> adw::HeaderBar {
        self.imp().headerbar.get()
    }

    pub(crate) fn left_close_button(&self) -> Button {
        self.imp().left_close_button.get()
    }

    pub(crate) fn filters_flow(&self) -> FlowBox {
        self.imp().filters_flow.get()
    }
    pub(crate) fn filters_flow_init(&self){
        let filters_flow=self.filters_flow();
        let inbox_filter = FilterPaneRow::new(FilterType::INBOX);
        let today_filter = FilterPaneRow::new(FilterType::TODAY);
        let scheduled_filter =FilterPaneRow::new(FilterType::SCHEDULED) ;
        let labels_filter = FilterPaneRow::new(FilterType::LABELS) ;
        let pinboard_filter = FilterPaneRow::new(FilterType::PINBOARD) ;
        let completed_filter = FilterPaneRow::new(FilterType::COMPLETED);
        filters_flow.append (&inbox_filter);
        filters_flow.append (&today_filter);
        filters_flow.append (&scheduled_filter);
        filters_flow.append (&labels_filter);
        filters_flow.append (&pinboard_filter);
        filters_flow.append (&completed_filter);
        inbox_filter.init();
        today_filter.init();
        scheduled_filter.init();
        labels_filter.init();
        pinboard_filter.init();
        completed_filter.init();
        // filters_flow.child_activated.connect ((child) => {
        //     let filter = (Layouts.FilterPaneRow) child;
        //     Services.EventBus.get_default ().pane_selected (PaneType.FILTER, filter.filter_type.to_string ());
        // });

    }
    pub(crate) fn right_close_button(&self) -> Button {
        self.imp().right_close_button.get()
    }

    pub(crate) fn appmenu(&self) -> RnAppMenu {
        self.imp().appmenu.get()
    }

    pub(crate) fn sidebar_stack(&self) -> adw::ViewStack {
        self.imp().sidebar_stack.get()
    }
    // ANCHOR_END: setup_callbacks
    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();
        
        imp.appmenu.get().init(appwindow);
        self.filters_flow().set_min_children_per_line(2);
        self.filters_flow().set_max_children_per_line(2);
        self.filters_flow_init();
        // imp.workspacebrowser.get().init(appwindow);
        // imp.settings_panel.get().init(appwindow);

        imp.left_close_button.connect_clicked(clone!(
            #[weak]
            appwindow,
            move |_| {
                appwindow.overlay_split_view().set_show_sidebar(false);
            }
        ));
        imp.right_close_button.connect_clicked(clone!(
            #[weak]
            appwindow,
            move |_| {
                appwindow.overlay_split_view().set_show_sidebar(false);
            }
        ));
    }
}
