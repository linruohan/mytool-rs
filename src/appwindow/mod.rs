// Modules
mod actions;
mod appsettings;
mod imp;

// Imports
use crate::{
    collection_object::CollectionObject, config, dialogs, task_object::TaskObject,
    FileType, RnApp, RnCanvasWrapper, RnMainHeader, RnOverlays, RnSidebar,
};
use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    gdk, gio, glib, glib::clone, Application, CustomFilter, FilterListModel, IconTheme,
    NoSelection,
};
use std::path::Path;
use tracing::{error, warn};

glib::wrapper! {
    pub(crate) struct RnAppWindow(ObjectSubclass<imp::RnAppWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl RnAppWindow {
    const AUTOSAVE_INTERVAL_DEFAULT: u32 = 30;
    const PERIODIC_CONFIGSAVE_INTERVAL: u32 = 10;

    pub(crate) fn new(app: &Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    #[allow(unused)]
    pub(crate) fn save_in_progress(&self) -> bool {
        self.property::<bool>("save-in-progress")
    }

    #[allow(unused)]
    pub(crate) fn set_save_in_progress(&self, save_in_progress: bool) {
        self.set_property("save-in-progress", save_in_progress.to_value());
    }

    #[allow(unused)]
    pub(crate) fn autosave(&self) -> bool {
        self.property::<bool>("autosave")
    }

    #[allow(unused)]
    pub(crate) fn set_autosave(&self, autosave: bool) {
        self.set_property("autosave", autosave.to_value());
    }

    #[allow(unused)]
    pub(crate) fn autosave_interval_secs(&self) -> u32 {
        self.property::<u32>("autosave-interval-secs")
    }

    #[allow(unused)]
    pub(crate) fn set_autosave_interval_secs(&self, autosave_interval_secs: u32) {
        self.set_property("autosave-interval-secs", autosave_interval_secs.to_value());
    }

    #[allow(unused)]
    pub(crate) fn righthanded(&self) -> bool {
        self.property::<bool>("righthanded")
    }

    #[allow(unused)]
    pub(crate) fn set_righthanded(&self, righthanded: bool) {
        self.set_property("righthanded", righthanded.to_value());
    }

    #[allow(unused)]
    pub(crate) fn respect_borders(&self) -> bool {
        self.property::<bool>("respect-borders")
    }

    pub(crate) fn app(&self) -> RnApp {
        self.application().unwrap().downcast::<RnApp>().unwrap()
    }

    pub(crate) fn overview(&self) -> adw::TabOverview {
        self.imp().overview.get()
    }

    pub(crate) fn main_header(&self) -> RnMainHeader {
        self.imp().main_header.get()
    }

    pub(crate) fn split_view(&self) -> adw::OverlaySplitView {
        self.imp().split_view.get()
    }

    pub(crate) fn sidebar(&self) -> RnSidebar {
        self.imp().sidebar.get()
    }

    pub(crate) fn overlays(&self) -> RnOverlays {
        self.imp().overlays.get()
    }

    /// Must be called after application is associated with the window else the init will panic
    pub(crate) fn init(&self) {
        let imp = self.imp();

        imp.overlays.get().init(self);
        imp.sidebar.get().init(self);
        imp.main_header.get().init(self);

        // An initial tab. Must! come before setting up the settings binds and import
        self.add_initial_tab();

        // actions and settings AFTER widget inits
        self.setup_icon_theme();
        self.setup_actions();
        self.setup_action_accels();

        if !self.app().settings_schema_found() {
            // Display an error toast if settings schema could not be found
            self.overlays().dispatch_toast_error(
                "Settings schema is not installed. App settings could not be loaded and won't be saved.",
            );
        } else {
            if let Err(e) = self.setup_settings_binds() {
                error!("Failed to setup settings binds, Err: {e:?}");
            }
            if let Err(e) = self.setup_periodic_save() {
                error!("Failed to setup periodic save, Err: {e:?}");
            }
            if let Err(e) = self.load_settings() {
                error!("Failed to load initial settings, Err: {e:?}");
            }
        }

        // Anything that needs to be done right before showing the appwindow

        // Set undo / redo as not sensitive as default - setting it in .ui file did not work for some reason

        if let Some(wrapper) = self.active_tab_wrapper() {
            self.refresh_ui_from_engine(&wrapper);
        }
    }

    fn setup_icon_theme(&self) {
        // add icon theme resource path because automatic lookup does not work in the devel build.
        let app_icon_theme =
            IconTheme::for_display(&<Self as gtk::prelude::WidgetExt>::display(self));
        app_icon_theme
            .add_resource_path((String::from(config::APP_IDPATH) + "icons").as_str());
    }

    /// Called to close the window
    pub(crate) fn close_force(&self) {
        if self.app().settings_schema_found() {
            // Saving all state
            if let Err(e) = self.save_to_settings() {
                error!("Failed to save appwindow to settings, Err: {e:?}");
            }
        }

        // Closing the state tasks channel receiver for all tabs
        for tab in self
            .tabs_snapshot()
            .into_iter()
            .map(|p| p.child().downcast::<RnCanvasWrapper>().unwrap())
        {
            // let _ = tab.canvas().engine_mut().set_active(false);
            // tab.canvas()
            //     .engine_ref()
            //     .engine_tasks_tx()
            //     .send(EngineTask::Quit);
        }

        self.destroy();
    }

    /// Get the active (selected) tab page.
    pub(crate) fn active_tab_page(&self) -> Option<adw::TabPage> {
        self.imp().overlays.tabview().selected_page()
    }

    pub(crate) fn n_tabs_open(&self) -> usize {
        self.imp().overlays.tabview().pages().n_items() as usize
    }

    /// Returns a vector of all tabs of the current windows
    pub(crate) fn get_all_tabs(&self) -> Vec<RnCanvasWrapper> {
        let n_tabs = self.n_tabs_open();
        let mut tabs = Vec::with_capacity(n_tabs);

        for i in 0..n_tabs {
            let wrapper = self
                .imp()
                .overlays
                .tabview()
                .pages()
                .item(i as u32)
                .unwrap()
                .downcast::<adw::TabPage>()
                .unwrap()
                .child()
                .downcast::<crate::RnCanvasWrapper>()
                .unwrap();
            tabs.push(wrapper);
        }
        tabs
    }

    /// Get the active (selected) tab page child.
    pub(crate) fn active_tab_wrapper(&self) -> Option<RnCanvasWrapper> {
        self.active_tab_page()
            .map(|c| c.child().downcast::<RnCanvasWrapper>().unwrap())
    }

    /// Get the active (selected) tab page canvas.
    // pub(crate) fn active_tab_canvas(&self) -> Option<RnCanvas> {
    //     self.active_tab_wrapper().map(|w| w.canvas())
    // }

    /// adds the initial tab to the tabview
    fn add_initial_tab(&self) -> adw::TabPage {
        let wrapper = RnCanvasWrapper::new();
        if let Some(app_settings) = self.app().app_settings() {
            // if let Err(e) = wrapper
            //     .canvas()
            //     .load_engine_config_from_settings(&app_settings)
            // {
            //     error!("Failed to load engine config for initial tab, Err: {e:?}");
            // }
        } else {
            warn!(
                "Could not load settings for initial tab. Settings schema not found."
            );
        }
        self.append_wrapper_new_tab(&wrapper)
    }

    /// Creates a new canvas wrapper without attaching it as a tab.
    pub(crate) fn new_canvas_wrapper(&self) -> RnCanvasWrapper {
        // let engine_config = self
        //     .active_tab_wrapper()
        //     .map(|w| w.canvas().engine_ref().extract_engine_config())
        //     .unwrap_or_default();
        let wrapper = RnCanvasWrapper::new();
        // let widget_flags = wrapper
        //     .canvas()
        //     .engine_mut()
        //     .load_engine_config(engine_config, crate::env::pkg_data_dir().ok());
        // self.handle_widget_flags(widget_flags, &wrapper.canvas());
        wrapper
    }

    /// Append the wrapper as a new tab and set it selected.
    pub(crate) fn append_wrapper_new_tab(
        &self,
        wrapper: &RnCanvasWrapper,
    ) -> adw::TabPage {
        // The tab page connections are handled in page_attached,
        // which is emitted when the page is added to the tabview.
        let page = self.overlays().tabview().append(wrapper);
        self.overlays().tabview().set_selected_page(&page);
        page
    }

    pub(crate) fn tabs_snapshot(&self) -> Vec<adw::TabPage> {
        self.overlays()
            .tabview()
            .pages()
            .snapshot()
            .into_iter()
            .map(|o| o.downcast::<adw::TabPage>().unwrap())
            .collect()
    }

    pub(crate) fn tabs_any_unsaved_changes(&self) -> bool {
        //     self.overlays()
        //         .tabview()
        //         .pages()
        //         .snapshot()
        //         .iter()
        //         .map(|o| {
        //             o.downcast_ref::<adw::TabPage>()
        //                 .unwrap()
        //                 .child()
        //                 .downcast_ref::<RnCanvasWrapper>()
        //                 .unwrap()
        //                 .canvas()
        //         })
        //         .any(|c| c.unsaved_changes())
        true
    }

    pub(crate) fn tabs_any_saves_in_progress(&self) -> bool {
        // self.overlays()
        //     .tabview()
        //     .pages()
        //     .snapshot()
        //     .iter()
        //     .map(|o| {
        //         o.downcast_ref::<adw::TabPage>()
        //             .unwrap()
        //             .child()
        //             .downcast_ref::<RnCanvasWrapper>()
        //             .unwrap()
        //             .canvas()
        //     })
        //     .any(|c| c.save_in_progress())
        true
    }

    /// Set all unselected tabs inactive.
    ///
    /// This clears the rendering and deinits the current pen of the engine in the tabs.
    ///
    /// To set a tab active again and reinit all necessary state, use `canvas.engine_mut().set_active(true)`.
    pub(crate) fn tabs_set_unselected_inactive(&self) {
        for inactive_page in self
            .overlays()
            .tabview()
            .pages()
            .snapshot()
            .into_iter()
            .map(|o| o.downcast::<adw::TabPage>().unwrap())
            .filter(|p| !p.is_selected())
        {
            // let canvas = inactive_page
            //     .child()
            //     .downcast::<RnCanvasWrapper>()
            //     .unwrap()
            //     .canvas();
            // // no need to handle the widget flags, since the tabs become inactive
            // let _ = canvas.engine_mut().set_active(false);
        }
    }

    /// Request to close the given tab.
    ///
    /// This must then be followed up by close_tab_finish() with confirm = true to close the tab,
    /// or confirm = false to revert.
    pub(crate) fn close_tab_request(&self, tab_page: &adw::TabPage) {
        self.overlays().tabview().close_page(tab_page);
    }

    /// Complete a close_tab_request.
    ///
    /// Closes the given tab when confirm is true, else reverts so that close_tab_request() can be called again.
    pub(crate) fn close_tab_finish(&self, tab_page: &adw::TabPage, confirm: bool) {
        self.overlays()
            .tabview()
            .close_page_finish(tab_page, confirm);
    }

    // pub(crate) fn refresh_titles(&self, canvas: &RnCanvas) {
    //     // Titles
    //     let title = canvas.doc_title_display();
    //     let subtitle = canvas.doc_folderpath_display();

    //     self.set_title(Some(
    //         &(title.clone() + " - " + config::APP_NAME_CAPITALIZED),
    //     ));

    //     self.main_header()
    //         .main_title_unsaved_indicator()
    //         .set_visible(canvas.unsaved_changes());
    //     if canvas.unsaved_changes() {
    //         self.main_header()
    //             .main_title()
    //             .add_css_class("unsaved_changes");
    //     } else {
    //         self.main_header()
    //             .main_title()
    //             .remove_css_class("unsaved_changes");
    //     }

    //     self.main_header().main_title().set_title(&title);
    //     self.main_header().main_title().set_subtitle(&subtitle);
    // }

    /// Open the file, with import dialogs when appropriate.
    ///
    /// When the file is a rnote save file, `rnote_file_new_tab` determines if a new tab is opened,
    /// or if it loads and overwrites the content of the current active one.
    pub(crate) async fn open_file_w_dialogs(
        &self,
        input_file: gio::File,
        target_pos: Option<na::Vector2<f64>>,
        rnote_file_new_tab: bool,
    ) {
        self.overlays().progressbar_start_pulsing();
        match self
            .try_open_file(input_file, target_pos, rnote_file_new_tab)
            .await
        {
            Ok(true) => {
                self.overlays().progressbar_finish();
            }
            Ok(false) => {
                self.overlays().progressbar_abort();
            }
            Err(e) => {
                error!("Opening file with dialogs failed, Err: {e:?}");

                self.overlays().dispatch_toast_error("Opening file failed");
                self.overlays().progressbar_abort();
            }
        }
    }

    /// Internal method for opening/importing content from a file with a supported content type.
    ///
    /// Returns Ok(true) if file was imported, Ok(false) if not, Err(_) if the import failed.
    async fn try_open_file(
        &self,
        input_file: gio::File,
        target_pos: Option<na::Vector2<f64>>,
        rnote_file_new_tab: bool,
    ) -> anyhow::Result<bool> {
        let file_imported = match FileType::lookup_file_type(&input_file) {
            FileType::MyToolFile => {
                let input_file_path = input_file.path().ok_or_else(|| {
                    anyhow::anyhow!(
                        "Could not open file '{input_file:?}', file path is None."
                    )
                })?;

                true
            }
            FileType::VectorImageFile => true,
            FileType::BitmapImageFile => true,
            FileType::XoppFile => true,
            FileType::PdfFile => true,
            FileType::PlaintextFile => true,
            FileType::Folder => false,
            FileType::Unsupported => {
                return Err(anyhow::anyhow!("Tried to open unsupported file type"));
            }
        };

        Ok(file_imported)
    }

    /// Refresh the UI from the engine state from the given tab page.
    pub(crate) fn refresh_ui_from_engine(&self, active_tab: &RnCanvasWrapper) {
        // self.sidebar().settings_panel().refresh_ui(active_tab);
        // self.refresh_titles(&canvas);
    }

    /// Sync the state from the previous active tab and the current one. Used when the selected tab changes.
    pub(crate) fn sync_state_between_tabs(
        &self,
        prev_tab: &adw::TabPage,
        active_tab: &adw::TabPage,
    ) {
        if *prev_tab == *active_tab {
            return;
        }
        // let prev_canvas_wrapper =
        //     prev_tab.child().downcast::<RnCanvasWrapper>().unwrap();
        // let prev_canvas = prev_canvas_wrapper.canvas();
        // let active_canvas_wrapper =
        //     active_tab.child().downcast::<RnCanvasWrapper>().unwrap();
        // let active_canvas = active_canvas_wrapper.canvas();

        // let mut widget_flags = active_canvas.engine_mut().load_engine_config_sync_tab(
        //     prev_canvas.engine_ref().extract_engine_config(),
        //     crate::env::pkg_data_dir().ok(),
        // );
        // // The visual-debug field is not saved in the config, but we want to sync its value between tabs.
        // widget_flags |= active_canvas
        //     .engine_mut()
        //     .set_visual_debug(prev_canvas.engine_mut().visual_debug());

        // self.handle_widget_flags(widget_flags, &active_canvas);
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
        let filter_state: String = self.app().app_settings()?.get("filter");

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
    pub(crate) fn set_current_collection(&self, collection: CollectionObject) {
        // Wrap model with filter and selection and pass it to the list box
        let tasks = collection.tasks();
        let filter_model = FilterListModel::new(Some(tasks.clone()), self.filter());
        let selection_model = NoSelection::new(Some(filter_model.clone()));
        let wrapper = RnCanvasWrapper::new();
        // wrapper.imp().tasks_list.bind_model(
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
        // if let Some(handler_id) = wrapper.imp().tasks_changed_handler_id.take() {
        //     self.tasks().disconnect(handler_id);
        // }

        // Assure that the task list is only visible when it is supposed to
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

        // // Set current tasks
        // self.imp().current_collection.replace(Some(collection));

        // self.select_collection_row();
    }
}
