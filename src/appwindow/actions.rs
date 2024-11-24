// Imports
use crate::{config, dialogs, RnAppWindow};
use gtk::{gio, glib, glib::clone, prelude::*, UriLauncher, Window};
use tracing::{debug, error};

const CLIPBOARD_INPUT_STREAM_BUFSIZE: usize = 4096;

impl RnAppWindow {
    /// Boolean actions have no target, and a boolean state. They have a default implementation for the activate signal,
    /// which requests the state to be inverted, and the default implementation for change_state, which sets the state
    /// to the request.
    ///
    /// We generally want to connect to the change_state signal. (but then have to set the state with
    /// `action.set_state()`)
    ///
    /// We can then either toggle the state through activating the action, or set the state explicitly through
    /// `action.change_state(<request>)`
    pub(crate) fn setup_actions(&self) {
        let action_fullscreen =
            gio::PropertyAction::new("fullscreen", self, "fullscreened");
        self.add_action(&action_fullscreen);
        let action_open_settings = gio::SimpleAction::new("open-settings", None);
        self.add_action(&action_open_settings);
        let action_about = gio::SimpleAction::new("about", None);
        self.add_action(&action_about);
        let action_donate = gio::SimpleAction::new("donate", None);
        self.add_action(&action_donate);
        let action_keyboard_shortcuts_dialog =
            gio::SimpleAction::new("keyboard-shortcuts", None);
        self.add_action(&action_keyboard_shortcuts_dialog);
        let action_open_appmenu = gio::SimpleAction::new("open-appmenu", None);
        self.add_action(&action_open_appmenu);
        let action_toggle_overview = gio::SimpleAction::new("toggle-overview", None);
        self.add_action(&action_toggle_overview);
        let action_devel_mode =
            gio::SimpleAction::new_stateful("devel-mode", None, &false.to_variant());
        self.add_action(&action_devel_mode);
        let action_devel_menu = gio::SimpleAction::new("devel-menu", None);
        self.add_action(&action_devel_menu);
        let action_new_tab = gio::SimpleAction::new("new-tab", None);
        self.add_action(&action_new_tab);
        let action_filter = gio::SimpleAction::new("filter", None);
        self.add_action(&action_filter);
        let action_visual_debug =
            gio::SimpleAction::new_stateful("visual-debug", None, &false.to_variant());
        self.add_action(&action_visual_debug);
        let action_righthanded =
            gio::PropertyAction::new("righthanded", self, "righthanded");
        self.add_action(&action_righthanded);
        let action_snap_positions = gio::SimpleAction::new_stateful(
            "snap-positions",
            None,
            &false.to_variant(),
        );
        self.add_action(&action_snap_positions);
        let action_show_format_borders = gio::SimpleAction::new_stateful(
            "show-format-borders",
            None,
            &true.to_variant(),
        );
        self.add_action(&action_show_format_borders);
        let action_show_origin_indicator = gio::SimpleAction::new_stateful(
            "show-origin-indicator",
            None,
            &true.to_variant(),
        );
        self.add_action(&action_show_origin_indicator);

        let action_respect_borders =
            gio::PropertyAction::new("respect-borders", self, "respect-borders");
        self.add_action(&action_respect_borders);

        let action_clipboard_paste_contextmenu =
            gio::SimpleAction::new("clipboard-paste-contextmenu", None);
        self.add_action(&action_clipboard_paste_contextmenu);
        let action_active_tab_move_left =
            gio::SimpleAction::new("active-tab-move-left", None);
        self.add_action(&action_active_tab_move_left);
        let action_active_tab_move_right =
            gio::SimpleAction::new("active-tab-move-right", None);
        self.add_action(&action_active_tab_move_right);
        let action_active_tab_close = gio::SimpleAction::new("active-tab-close", None);
        self.add_action(&action_active_tab_close);

        // Open settings
        action_open_settings.connect_activate(clone!(
            #[weak(rename_to = appwindow)]
            self,
            move |_, _| {
                appwindow
                    .sidebar()
                    .sidebar_stack()
                    .set_visible_child_name("settings_page");
                appwindow.split_view().set_show_sidebar(true);
            }
        ));

        // About Dialog
        action_about.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                dialogs::dialog_about(&appwindow);
            }
        ));

        // Donate
        action_donate.connect_activate(clone!(move |_, _| {
            UriLauncher::new(config::APP_DONATE_URL).launch(
                None::<&Window>,
                gio::Cancellable::NONE,
                |res| {
                    if let Err(e) = res {
                        error!("Launching donate URL failed, Err: {e:?}");
                    }
                },
            )
        }));

        // Keyboard shortcuts
        action_keyboard_shortcuts_dialog.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                dialogs::dialog_keyboard_shortcuts(&appwindow);
            }
        ));

        // Open App Menu
        action_open_appmenu.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                if !appwindow.split_view().shows_sidebar() {
                    appwindow.main_header().appmenu().popovermenu().popup();
                    return;
                }
                if appwindow.split_view().is_collapsed() {
                    appwindow.split_view().set_show_sidebar(false);
                    appwindow.main_header().appmenu().popovermenu().popup();
                } else {
                    appwindow.sidebar().appmenu().popovermenu().popup();
                }
            }
        ));

        // Toggle Tabs Overview
        action_toggle_overview.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let overview = appwindow.overview();
                overview.set_open(!overview.is_open());
            }
        ));

        // Developer mode
        action_devel_mode.connect_activate(clone!(
            #[weak]
            action_devel_menu,
            #[weak]
            action_visual_debug,
            move |action, _| {
                let state = action.state().unwrap().get::<bool>().unwrap();

                // Enable the devel menu action to reveal it in the app menu
                action_devel_menu.set_enabled(!state);

                // Always disable visual-debugging when disabling the developer mode
                if state {
                    debug!("Disabling developer mode, disabling visual debugging.");
                    action_visual_debug.change_state(&false.to_variant());
                }
                action.change_state(&(!state).to_variant());
            }
        ));

        // Developer settings
        // Its enabled state toggles the visibility of the developer settings menu entry.
        // Must only be modified inside the devel-mode action
        action_devel_menu.set_enabled(false);

        // Create page
        action_new_tab.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let wrapper = appwindow.new_canvas_wrapper();
                appwindow.append_wrapper_new_tab(&wrapper);
            }
        ));

        // Tab actions
        action_active_tab_move_left.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(active_tab_page) = appwindow.active_tab_page() else {
                    return;
                };
                appwindow
                    .overlays()
                    .tabview()
                    .reorder_backward(&active_tab_page);
            }
        ));
        action_active_tab_move_right.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(active_tab_page) = appwindow.active_tab_page() else {
                    return;
                };
                appwindow
                    .overlays()
                    .tabview()
                    .reorder_forward(&active_tab_page);
            }
        ));
        action_active_tab_close.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                let Some(active_tab_page) = appwindow.active_tab_page() else {
                    return;
                };
                if appwindow.overlays().tabview().n_pages() <= 1 {
                    // If there is only one tab left, request to close the entire window.
                    appwindow.close();
                } else {
                    appwindow.close_tab_request(&active_tab_page);
                }
            }
        ));
    }

    pub(crate) fn setup_action_accels(&self) {
        let app = self.app();

        app.set_accels_for_action("win.active-tab-close", &["<Ctrl>w"]);
        app.set_accels_for_action("win.fullscreen", &["F11"]);
        app.set_accels_for_action("win.keyboard-shortcuts", &["<Ctrl>question"]);
        app.set_accels_for_action("win.toggle-overview", &["<Ctrl><Shift>o"]);
        app.set_accels_for_action("win.open-appmenu", &["F10"]);

        app.set_accels_for_action("win.new-tab", &["<Ctrl>t"]);
        app.set_accels_for_action("win.snap-positions", &["<Ctrl><Shift>p"]);
        // tasks
        app.set_accels_for_action("win.filter('All')", &["<Ctrl>a"]);
        app.set_accels_for_action("win.filter('Open')", &["<Ctrl>o"]);
        app.set_accels_for_action("win.filter('Done')", &["<Ctrl>d"]);
        // shortcuts for devel build
        if config::PROFILE.to_lowercase().as_str() == "devel" {
            app.set_accels_for_action("win.visual-debug", &["<Ctrl><Shift>v"]);
        }
    }
}
