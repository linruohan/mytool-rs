// Imports
use crate::collection_object::{CollectionData, CollectionObject};
use crate::task_object::TaskObject;
use crate::{RnAppMenu, RnAppWindow};
use adw::AlertDialog;
use adw::{prelude::*, subclass::prelude::*, ResponseAppearance};
use gtk::CustomFilter;
use gtk::{
    gio, glib, glib::clone, pango, Button, CompositeTemplate, Entry, FilterListModel,
    Label, ListBox, ListBoxRow, NoSelection, Widget,
};
use std::cell::{OnceCell, RefCell};
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
        pub(crate) collections_list: TemplateChild<ListBox>,
        pub collections: OnceCell<gio::ListStore>,
        pub current_collection: RefCell<Option<CollectionObject>>,
        pub current_filter_model: RefCell<Option<FilterListModel>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for RnSidebar {
        const NAME: &'static str = "RnSidebar";
        type Type = super::RnSidebar;
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

    pub(crate) fn right_close_button(&self) -> Button {
        self.imp().right_close_button.get()
    }

    pub(crate) fn appmenu(&self) -> RnAppMenu {
        self.imp().appmenu.get()
    }

    pub(crate) fn sidebar_stack(&self) -> adw::ViewStack {
        self.imp().sidebar_stack.get()
    }

    // pub(crate) fn workspacebrowser(&self) -> RnWorkspaceBrowser {
    //     self.imp().workspacebrowser.get()
    // }

    // pub(crate) fn settings_panel(&self) -> RnSettingsPanel {
    //     self.imp().settings_panel.get()
    // }
    // ANCHOR: helper
    pub(crate) fn tasks(&self) -> gio::ListStore {
        self.current_collection().tasks()
    }
    pub(crate) fn current_collection(&self) -> CollectionObject {
        self.imp()
            .current_collection
            .borrow()
            .clone()
            .expect("`current_collection` should be set in `set_current_collections`.")
    }

    pub(crate) fn collections(&self) -> gio::ListStore {
        self.imp()
            .collections
            .get()
            .expect("`collections` should be set in `setup_collections`.")
            .clone()
    }
    // ANCHOR: setup_collections
    pub(crate) fn setup_collections(&self) {
        let collections = gio::ListStore::new::<CollectionObject>();
        self.imp()
            .collections
            .set(collections.clone())
            .expect("Could not set collections");

        self.imp().collections_list.bind_model(
            Some(&collections),
            clone!(
                #[weak(rename_to = window)]
                self,
                #[upgrade_or_panic]
                move |obj| {
                    let collection_object = obj
                        .downcast_ref()
                        .expect("The object should be of type `CollectionObject`.");
                    let row = window.create_collection_row(collection_object);
                    row.upcast()
                }
            ),
        )
    }
    // ANCHOR: create_collection_row
    pub(crate) fn create_collection_row(
        &self,
        collection_object: &CollectionObject,
    ) -> ListBoxRow {
        let label = Label::builder()
            .ellipsize(pango::EllipsizeMode::End)
            .xalign(0.0)
            .build();

        collection_object
            .bind_property("title", &label, "label")
            .sync_create()
            .build();

        ListBoxRow::builder().child(&label).build()
    }
    fn remove_done_tasks(&self) {
        let tasks = self.tasks();
        let mut position = 0;
        while let Some(item) = tasks.item(position) {
            // Get `TaskObject` from `glib::Object`
            let task_object = item
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            if task_object.is_completed() {
                tasks.remove(position);
            } else {
                position += 1;
            }
        }
    }
    async fn new_collection(&self) {
        // Create entry
        let entry = Entry::builder()
            .placeholder_text("Name")
            .activates_default(true)
            .build();

        let cancel_response = "cancel";
        let create_response = "create";

        // Create new dialog
        let dialog = AlertDialog::builder()
            .heading("New Collection")
            .close_response(cancel_response)
            .default_response(create_response)
            .extra_child(&entry)
            .build();
        dialog
            .add_responses(&[(cancel_response, "Cancel"), (create_response, "Create")]);
        // Make the dialog button insensitive initially
        dialog.set_response_enabled(create_response, false);
        dialog.set_response_appearance(create_response, ResponseAppearance::Suggested);

        // Set entry's css class to "error", when there is no text in it
        entry.connect_changed(clone!(
            #[weak]
            dialog,
            move |entry| {
                let text = entry.text();
                let empty = text.is_empty();

                dialog.set_response_enabled(create_response, !empty);

                if empty {
                    entry.add_css_class("error");
                } else {
                    entry.remove_css_class("error");
                }
            }
        ));

        let response = dialog.choose_future(self).await;

        // Return if the user chose `cancel_response`
        if response == cancel_response {
            return;
        }

        // Create a new list store
        let tasks = gio::ListStore::new::<TaskObject>();

        // Create a new collection object from the title the user provided
        let title = entry.text().to_string();
        let collection = CollectionObject::new(&title, tasks);

        // Add new collection object and set current tasks
        self.collections().append(&collection);
        // self.set_current_collection(collection);

        // Show the content
        // self.imp().split_view.set_show_content(true);
    }
    fn set_current_collection(&self, collection: CollectionObject) {
        // Wrap model with filter and selection and pass it to the list box
        let tasks = collection.tasks();
        let filter_model = FilterListModel::new(Some(tasks.clone()), self.filter());
        let selection_model = NoSelection::new(Some(filter_model.clone()));
        // self.imp().tasks_list.bind_model(
        //     Some(&selection_model),
        //     clone!(
        //         #[weak(rename_to = window)]
        //         self,
        //         #[upgrade_or_panic]
        //         move |obj| {
        //             let task_object = obj
        //                 .downcast_ref()
        //                 .expect("The object should be of type `TaskObject`.");
        //             let row = window.create_task_row(task_object);
        //             row.upcast()
        //         }
        //     ),
        // );

        // Store filter model
        self.imp().current_filter_model.replace(Some(filter_model));

        // If present, disconnect old `tasks_changed` handler
        // if let Some(handler_id) = self.imp().tasks_changed_handler_id.take() {
        //     self.tasks().disconnect(handler_id);
        // }

        // // Assure that the task list is only visible when it is supposed to
        // self.set_task_list_visible(&tasks);
        // let tasks_changed_handler_id = tasks.connect_items_changed(clone!(
        //     #[weak(rename_to = window)]
        //     self,
        //     move |tasks, _, _, _| {
        //         window.set_task_list_visible(tasks);
        //     }
        // ));
        // self.imp()
        //     .tasks_changed_handler_id
        //     .replace(Some(tasks_changed_handler_id));

        // Set current tasks
        self.imp().current_collection.replace(Some(collection));

        self.select_collection_row();
    }
    fn select_collection_row(&self) {
        if let Some(index) = self.collections().find(&self.current_collection()) {
            let row = self.imp().collections_list.row_at_index(index as i32);
            self.imp().collections_list.select_row(row.as_ref());
        }
    }

    fn set_filter(&self) {
        self.imp()
            .current_filter_model
            .borrow()
            .clone()
            .expect("`current_filter_model` should be set in `set_current_collection`.")
            .set_filter(self.filter().as_ref());
    }
    // ANCHOR_END: helper

    fn filter(&self) -> Option<CustomFilter> {
        // Get filter state from settings
        let filter_state: String = "All".to_owned();

        // Create custom filters
        let filter_open = CustomFilter::new(|obj| {
            // Get `TaskObject` from `glib::Object`
            let task_object = obj
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            // Only allow completed tasks
            !task_object.is_completed()
        });
        let filter_done = CustomFilter::new(|obj| {
            // Get `TaskObject` from `glib::Object`
            let task_object = obj
                .downcast_ref::<TaskObject>()
                .expect("The object needs to be of type `TaskObject`.");

            // Only allow done tasks
            task_object.is_completed()
        });

        // Return the correct filter
        match filter_state.as_str() {
            "All" => None,
            "Open" => Some(filter_open),
            "Done" => Some(filter_done),
            _ => unreachable!(),
        }
    }

    // ANCHOR_END: setup_callbacks
    pub(crate) fn init(&self, appwindow: &RnAppWindow) {
        let imp = self.imp();

        imp.appmenu.get().init(appwindow);
        // imp.workspacebrowser.get().init(appwindow);
        // imp.settings_panel.get().init(appwindow);
        self.setup_collections();

        imp.left_close_button.connect_clicked(clone!(
            #[weak]
            appwindow,
            move |_| {
                appwindow.split_view().set_show_sidebar(false);
            }
        ));
        imp.right_close_button.connect_clicked(clone!(
            #[weak]
            appwindow,
            move |_| {
                appwindow.split_view().set_show_sidebar(false);
            }
        ));
        self.collections().connect_items_changed(clone!(
            #[weak(rename_to = window)]
            self,
            move |_, _, _, _| {
                // window.set_stack();
            }
        ));

        // Setup callback for activating a row of collections list
        // imp.collections_list.get().connect_row_activated(clone!(
        //     #[weak(rename_to = window)]
        //     self,
        //     move |_, row| {
        //         let index = row.index();
        //         let selected_collection = window
        //             .collections()
        //             .item(index as u32)
        //             .expect("There needs to be an object at this position.")
        //             .downcast::<CollectionObject>()
        //             .expect("The object needs to be a `CollectionObject`.");
        //         appwindow.set_current_collection(selected_collection);
        //         // window.imp().split_view.set_show_content(true);
        //     }
        // ));
    }
}
