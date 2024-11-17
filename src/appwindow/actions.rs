// Imports
use crate::{config, dialogs, RnAppWindow};
use gtk::{gio, glib, glib::clone, prelude::*, UriLauncher, Window};
use std::path::PathBuf;
use tracing::{debug, error};

const CLIPBOARD_INPUT_STREAM_BUFSIZE: usize = 4096;

impl RnAppWindow {
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
        let action_new_tab = gio::SimpleAction::new("new-tab", None);
        self.add_action(&action_new_tab);
        let action_righthanded =
            gio::PropertyAction::new("righthanded", self, "righthanded");
        self.add_action(&action_righthanded);
        let action_clipboard_paste = gio::SimpleAction::new("clipboard-paste", None);
        self.add_action(&action_clipboard_paste);
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

        // Clipboard paste
        action_clipboard_paste.connect_activate(clone!(
            #[weak(rename_to=appwindow)]
            self,
            move |_, _| {
                appwindow.clipboard_paste(None);
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
        app.set_accels_for_action("win.clipboard-paste", &["<Ctrl>v"]);

        // shortcuts for devel build
        if config::PROFILE.to_lowercase().as_str() == "devel" {
            app.set_accels_for_action("win.visual-debug", &["<Ctrl><Shift>v"]);
        }
    }

    fn clipboard_paste(&self, target_pos: Option<na::Vector2<f64>>) {
        let content_formats = self.clipboard().formats();

        // Order matters here, we want to go from specific -> generic, mostly because `text/plain` is contained in other text based formats
        if content_formats.contain_mime_type("text/uri-list") {
            glib::spawn_future_local(clone!(
                #[weak(rename_to=appwindow)]
                self,
                async move {
                    debug!("Recognized clipboard content format: files list");

                    match appwindow.clipboard().read_text_future().await {
                        Ok(Some(text)) => {
                            let file_paths = text
                                .lines()
                                .filter_map(|line| {
                                    let file_path =
                                        if let Ok(path_uri) = url::Url::parse(line) {
                                            path_uri.to_file_path().ok()?
                                        } else {
                                            PathBuf::from(&line)
                                        };

                                    if file_path.exists() {
                                        Some(file_path)
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<PathBuf>>();

                            for file_path in file_paths {
                                appwindow
                                    .open_file_w_dialogs(
                                        gio::File::for_path(&file_path),
                                        target_pos,
                                        true,
                                    )
                                    .await;
                            }
                        }
                        Ok(None) => {}
                        Err(e) => {
                            error!("Reading clipboard text while pasting clipboard from path failed, Err: {e:?}");
                        }
                    }
                }
            ));
        } else if content_formats.contain_mime_type("image/svg+xml") {
            glib::spawn_future_local(clone!(
                #[weak(rename_to=appwindow)]
                self,
                async move {
                    debug!("Recognized clipboard content: svg image");

                    match appwindow
                        .clipboard()
                        .read_future(
                            &["image/svg+xml"],
                            glib::source::Priority::DEFAULT,
                        )
                        .await
                    {
                        Ok((input_stream, _)) => {
                            let mut acc = Vec::new();
                            loop {
                                match input_stream
                                    .read_future(
                                        vec![0; CLIPBOARD_INPUT_STREAM_BUFSIZE],
                                        glib::source::Priority::DEFAULT,
                                    )
                                    .await
                                {
                                    Ok((mut bytes, n)) => {
                                        if n == 0 {
                                            break;
                                        }
                                        acc.append(&mut bytes);
                                    }
                                    Err(e) => {
                                        error!("Failed to read clipboard input stream while pasting as Svg, Err: {e:?}");
                                        acc.clear();
                                        break;
                                    }
                                }
                            }

                            if !acc.is_empty() {
                                match crate::utils::str_from_u8_nul_utf8(&acc) {
                                Ok(text) => {
                                    error!(
                                        "error {text}"
                                    );
                                }
                                Err(e) => error!("Failed to get string from clipboard data while pasting as Svg, Err: {e:?}"),
                            }
                            }
                        }
                        Err(e) => {
                            error!(
                                "Failed to read clipboard data while pasting as Svg, Err: {e:?}"
                            );
                        }
                    };
                }
            ));
        } else if content_formats.contain_mime_type("image/png")
            || content_formats.contain_mime_type("image/jpeg")
            || content_formats.contain_mime_type("image/jpg")
            || content_formats.contain_mime_type("image/tiff")
            || content_formats.contain_mime_type("image/bmp")
        {
            const MIMES: [&str; 5] = [
                "image/png",
                "image/jpeg",
                "image/jpg",
                "image/tiff",
                "image/bmp",
            ];
            if let Some(mime_type) = MIMES
                .into_iter()
                .find(|&mime| content_formats.contain_mime_type(mime))
            {
                error!(
                    "Loading bitmap image bytes failed while pasting clipboard as {mime_type}"
                );
            }
        } else if content_formats.contain_mime_type("text/plain")
            || content_formats.contain_mime_type("text/plain;charset=utf-8")
        {
        } else {
            debug!(
                "Failed to paste clipboard, unsupported MIME-type(s): {:?}",
                content_formats.mime_types()
            );
        }
    }
}
