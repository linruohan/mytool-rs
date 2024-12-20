// Modules
mod actions;
mod appsettings;
mod imp;

// Imports
use crate::{config, RnApp, RnSidebar};
use adw::{prelude::*, subclass::prelude::*, ViewStack};
use gtk::{gio, glib, glib::clone, Application, IconTheme};
use tracing::error;

glib::wrapper! {
    pub(crate) struct RnAppWindow(ObjectSubclass<imp::RnAppWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl RnAppWindow {
    pub(crate) fn new(app: &Application) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    #[allow(unused)]
    pub(crate) fn righthanded(&self) -> bool {
        self.property::<bool>("righthanded")
    }

    #[allow(unused)]
    pub(crate) fn set_righthanded(&self, righthanded: bool) {
        self.set_property("righthanded", righthanded.to_value());
    }

    pub(crate) fn app(&self) -> RnApp {
        self.application().unwrap().downcast::<RnApp>().unwrap()
    }
    pub(crate) fn main_header(&self) -> crate::RnMainHeader {
        self.imp().main_header.get()
    }
    pub(crate) fn todo(&self) -> crate::RnTodo {
        self.imp().todo.get()
    }
    #[allow(unused)]
    pub(crate) fn view_stack(&self) -> ViewStack {
        self.imp().view_stack.get()
    }

    pub(crate) fn overlay_split_view(&self) -> adw::OverlaySplitView {
        self.imp().overlay_split_view.get()
    }

    pub(crate) fn sidebar(&self) -> RnSidebar {
        self.imp().sidebar.get()
    }
    pub(crate) fn views_stack(&self) -> ViewStack {
        self.imp().views_stack.get()
    }
    /// Must be called after application is associated with the window else the init will panic
    pub(crate) fn init(&self) {
        let imp = self.imp();

        imp.sidebar.get().init(self);
        imp.main_header.get().init(self);
        // actions and settings AFTER widget inits
        self.setup_icon_theme();
        self.setup_actions();
        self.setup_action_accels();

        if !self.app().settings_schema_found() {
            // Display an error toast if settings schema could not be found
            // self.overlays().dispatch_toast_error(
            //     "Settings schema is not installed. App settings could not be loaded and won't be saved.",
            // );
        } else {
            if let Err(e) = self.setup_settings_binds() {
                error!("Failed to setup settings binds, Err: {e:?}");
            }
            if let Err(e) = self.load_settings() {
                error!("Failed to load initial settings, Err: {e:?}");
            }
        }

        // Anything that needs to be done right before showing the appwindow

        // Set undo / redo as not sensitive as default - setting it in .ui file did not work for some reason

        // if let Some(wrapper) = self.active_tab_wrapper() {
        //     self.refresh_ui_from_engine(&wrapper);
        // }
        self.imp()
            .views_stack
            .get()
            .connect_visible_child_name_notify(clone!(
                #[weak(rename_to=appwindow)]
                self,
                move |views_stack| {
                    if let Some(child_name) = views_stack.visible_child_name() {
                        match child_name.to_value().get::<String>().unwrap().as_str() {
                            "workspacebrowser_page" => {
                                appwindow
                                    .views_stack()
                                    .set_visible_child_name("workspacebrowser_page");
                            }
                            "done_page" => {
                                appwindow
                                    .views_stack()
                                    .set_visible_child_name("done_page");
                            }
                            _ => {}
                        };
                    };
                }
            ));
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

        self.destroy();
    }
}
