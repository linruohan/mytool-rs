// Imports
use crate::appwindow::RnAppWindow;
use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    gio, glib, CompositeTemplate, MenuButton, PopoverMenu, ToggleButton, Widget,
};

mod imp {
    use super::*;

    #[derive(Default, Debug, CompositeTemplate)]
    #[template(resource = "/com/github/linruohan/mytool/ui/appmenu.ui")]
    pub(crate) struct FilterPaneRow {
        // 标题栏的设置按钮
        #[template_child]
        pub(crate) menubutton: TemplateChild<MenuButton>,
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

            self.menubutton
                .get()
                .set_popover(Some(&self.popovermenu.get()));
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

impl Default for FilterPaneRow {
    fn default() -> Self {
        Self::new()
    }
}

impl FilterPaneRow {
    pub(crate) fn new() -> Self {
        glib::Object::new()
    }

    pub(crate) fn popovermenu(&self) -> PopoverMenu {
        self.imp().popovermenu.get()
    }

    pub(crate) fn init(&self, appwindow: &RnAppWindow) {}
}
