// Imports
use crate::appwindow::RnAppWindow;
use adw::prelude::*;
use gtk::glib::{self, clone};
use tracing::error;

impl RnAppWindow {
    /// Setup settings binds.
    pub(crate) fn setup_settings_binds(&self) -> anyhow::Result<()> {
        let app = self.app();
        let app_settings = app
            .app_settings()
            .ok_or_else(|| anyhow::anyhow!("Settings schema not found."))?;

        app.style_manager().connect_color_scheme_notify(clone!(
            #[weak]
            app_settings,
            move |style_manager| {
                let color_scheme = match style_manager.color_scheme() {
                    adw::ColorScheme::Default => String::from("default"),
                    adw::ColorScheme::ForceLight => String::from("force-light"),
                    adw::ColorScheme::ForceDark => String::from("force-dark"),
                    _ => String::from("default"),
                };

                if let Err(e) = app_settings.set_string("color-scheme", &color_scheme) {
                    error!("Failed to set setting `color-scheme`, Err: {e:?}");
                }
            }
        ));
        // filter
        let action_filter = app_settings.create_action("filter");
        self.add_action(&action_filter);
        app_settings.connect_changed(
            Some("filter"),
            clone!(
                #[weak(rename_to = window)]
                self,
                move |_, _| {
                    window.todo().set_filter();
                }
            ),
        );
        app_settings
            .bind("sidebar-show", &self.overlay_split_view(), "show-sidebar")
            .get_no_changes()
            .build();

        // righthanded
        app_settings
            .bind("righthanded", self, "righthanded")
            .get_no_changes()
            .build();

        Ok(())
    }

    /// Load settings that are not bound as binds.
    ///
    /// Settings changes through gsettings / dconf might not be applied until the app restarts.
    pub(crate) fn load_settings(&self) -> anyhow::Result<()> {
        let app = self.app();
        let app_settings = app
            .app_settings()
            .ok_or_else(|| anyhow::anyhow!("Settings schema not found."))?;

        // appwindow
        {
            let window_width = app_settings.int("window-width");
            let window_height = app_settings.int("window-height");
            let is_maximized = app_settings.boolean("is-maximized");

            if is_maximized {
                self.maximize();
            } else {
                self.set_default_size(window_width, window_height);
            }

            // set the color-scheme through the action
            let color_scheme = app_settings.string("color-scheme");
            self.app()
                .activate_action("color-scheme", Some(&color_scheme.to_variant()));
        }

        // {
        //     // Workspaces bar
        //     self.sidebar()
        //         .workspacebrowser()
        //         .workspacesbar()
        //         .load_from_settings(&app_settings);
        // }

        Ok(())
    }

    /// Save settings that are not bound as binds.
    pub(crate) fn save_to_settings(&self) -> anyhow::Result<()> {
        let app = self.app();
        let app_settings = app
            .app_settings()
            .ok_or_else(|| anyhow::anyhow!("Settings schema not found."))?;

        {
            // Appwindow
            app_settings.set_boolean("is-maximized", self.is_maximized())?;
            if !self.is_maximized() {
                app_settings.set_int("window-width", self.width())?;
                app_settings.set_int("window-height", self.height())?;
            }
        }

        // {
        //     // Workspaces list
        //     self.sidebar()
        //         .workspacebrowser()
        //         .workspacesbar()
        //         .save_to_settings(&app_settings);
        // }

        Ok(())
    }
}
