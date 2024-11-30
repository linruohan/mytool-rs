// Imports
use crate::{appwindow::RnAppWindow, task_object::TaskObject, RnTodo};
use adw::{prelude::*, subclass::prelude::*};
use gtk::{
    glib::{self, clone},
    CustomFilter,
};
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
        app_settings.connect_changed(
            Some("filter"),
            clone!(
                #[weak(rename_to = window)]
                self,
                move |_, _| {
                    window.set_filter();
                }
            ),
        );
        app_settings
            .bind("sidebar-show", &self.split_view(), "show-sidebar")
            .get_no_changes()
            .build();

        // autosave
        app_settings
            .bind("autosave", self, "autosave")
            .get_no_changes()
            .build();

        // autosave interval secs
        app_settings
            .bind("autosave-interval-secs", self, "autosave-interval-secs")
            .get_no_changes()
            .build();

        // righthanded
        app_settings
            .bind("righthanded", self, "righthanded")
            .get_no_changes()
            .build();

        // respect borders
        app_settings
            .bind("respect-borders", self, "respect-borders")
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
        //     // Save engine config of the current active tab
        //     if let Some(canvas) = self.active_tab_canvas() {
        //         canvas.save_engine_config(&app_settings)?;
        //     }
        // }

        // {
        //     // Workspaces list
        //     self.sidebar()
        //         .workspacebrowser()
        //         .workspacesbar()
        //         .save_to_settings(&app_settings);
        // }

        Ok(())
    }

    pub(crate) fn setup_periodic_save(&self) -> anyhow::Result<()> {
        let app = self.app();
        let app_settings = app
            .app_settings()
            .ok_or_else(|| anyhow::anyhow!("Settings schema not found."))?;

        if let Some(removed_id) = self
            .imp()
            .periodic_configsave_source_id
            .borrow_mut()
            .replace(glib::source::timeout_add_seconds_local(
                Self::PERIODIC_CONFIGSAVE_INTERVAL,
                clone!(
                    #[weak]
                    app_settings,
                    #[weak(rename_to=appwindow)]
                    self,
                    #[upgrade_or]
                    glib::ControlFlow::Break,
                    move || {
                        // let Some(canvas) = appwindow.active_tab_canvas() else {
                        //     return glib::ControlFlow::Continue;
                        // };
                        // if let Err(e) = canvas.save_engine_config(&app_settings) {
                        //     error!(
                        //         "Saving engine config in periodic save task failed , Err: {e:?}"
                        //     );
                        // }

                        glib::ControlFlow::Continue
                    }
                ),
            ))
        {
            removed_id.remove();
        }

        Ok(())
    }
}
