use crate::collection_object::{CollectionData, CollectionObject};
use crate::utils::data_path;
use adw::subclass::prelude::*;
use adw::{prelude::*, NavigationSplitView};
use glib::subclass::InitializingObject;
use gtk::glib::SignalHandlerId;
use gtk::{
    gio, glib, CompositeTemplate, Entry, FilterListModel, ListBox, Stack, Widget,
};
use std::cell::OnceCell;
use std::cell::RefCell;
use std::fs::File;

// ANCHOR: struct
// Object holding the state
#[derive(Debug, CompositeTemplate)]
#[template(resource = "/com/github/linruohan/mytool/ui/todo.ui")]
pub(crate) struct RnTodo {
    #[template_child]
    pub entry: TemplateChild<Entry>,
    #[template_child]
    pub tasks_list: TemplateChild<ListBox>,
    // 👇 all members below are new
    #[template_child]
    pub collections_list: TemplateChild<ListBox>,
    #[template_child]
    pub split_view: TemplateChild<NavigationSplitView>,
    #[template_child]
    pub stack: TemplateChild<Stack>,
    pub collections: OnceCell<gio::ListStore>,
    pub current_collection: RefCell<Option<CollectionObject>>,
    pub current_filter_model: RefCell<Option<FilterListModel>>,
    pub tasks_changed_handler_id: RefCell<Option<SignalHandlerId>>,
}
// ANCHOR_END: struct
impl Default for RnTodo {
    fn default() -> Self {
        Self {
            entry: TemplateChild::<Entry>::default(),
            tasks_list: TemplateChild::<ListBox>::default(),
            // 👇 all members below are new
            collections_list: TemplateChild::<ListBox>::default(),
            split_view: TemplateChild::<NavigationSplitView>::default(),
            stack: TemplateChild::<Stack>::default(),
            collections: OnceCell::<gio::ListStore>::default(),
            current_collection: RefCell::<Option<CollectionObject>>::default(),
            current_filter_model: RefCell::<Option<FilterListModel>>::default(),
            tasks_changed_handler_id: RefCell::<Option<SignalHandlerId>>::default(),
        }
    }
}
// ANCHOR: object_subclass
// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for RnTodo {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "RnTodo";
    type Type = super::RnTodo;
    type ParentType = Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();

        // Create action to remove done tasks and add to action group "win"
        klass.install_action("win.remove-done-tasks", None, |window, _, _| {
            window.remove_done_tasks();
        });

        // Create async action to create new collection and add to action group "win"
        klass.install_action_async(
            "win.new-collection",
            None,
            |window, _, _| async move {
                window.new_collection().await;
            },
        );
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}
// ANCHOR_END: object_subclass

// ANCHOR: object_impl
// Trait shared by all GObjects
impl ObjectImpl for RnTodo {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();

        // Setup
        let obj: glib::BorrowedObject<'_, super::RnTodo> = self.obj();
        obj.setup_collections();
        obj.restore_data();
        obj.setup_callbacks();
    }
}
// ANCHOR_END: object_impl

// Trait shared by all widgets
impl WidgetImpl for RnTodo {}
