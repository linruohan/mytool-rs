// Imports
use crate::FilterType;
use adw::{prelude::*, subclass::prelude::*};
use gtk::{gio::Icon, glib, CompositeTemplate, Image, Label, Widget};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/linruohan/mytool/ui/filter_pane_row.ui")]
    pub(crate) struct FilterPaneRow {
        pub(crate) filter_type: Option<FilterType>,
        // 标题栏的设置按钮
        #[template_child]
        title_image: TemplateChild<Image>,
        #[template_child]
        count_label: TemplateChild<Label>,
        #[template_child]
        title_label: TemplateChild<Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FilterPaneRow {
        const NAME: &'static str = "FilterPaneRow";
        type Type = super::FilterPaneRow;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for FilterPaneRow {
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

    impl WidgetImpl for FilterPaneRow {}
}

glib::wrapper! {
    pub(crate) struct FilterPaneRow(ObjectSubclass<imp::FilterPaneRow>)
    @extends Widget;
}

impl FilterPaneRow {
    pub(crate) fn new(filter: FilterType) -> Self {
        // glib::Object::new()
        let filter_type = filter;
        Self { filter_type }
    }
    pub(crate) fn filter_type(&self) -> FilterType {
        self.imp().filter_type.get()
    }
    pub(crate) fn init(&self) {
        let icon = Icon::for_string(&self.filter_type().get_icon())
            .expect("should be a valid Icon");
        // self.imp().title_image.get().set_gicon(&icon);
    }
}
