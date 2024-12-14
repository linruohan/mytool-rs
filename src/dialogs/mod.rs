// Imports
use crate::appwindow::RnAppWindow;
use crate::config;
use crate::globals;
use adw::prelude::*;
use gtk::{gio, glib, glib::clone, Builder, ShortcutsWindow};
use tracing::error;
// About Dialog
pub(crate) fn dialog_about(appwindow: &RnAppWindow) {
    let app_icon_name = if config::PROFILE == "devel" {
        config::APP_NAME.to_string() + "-devel"
    } else {
        config::APP_NAME.to_string()
    };

    let aboutdialog = adw::AboutDialog::builder()
        .application_name(config::APP_NAME_CAPITALIZED)
        .application_icon(app_icon_name)
        // .comments(gettext("Sketch and take handwritten notes"))
        .website(config::APP_WEBSITE)
        .issue_url(config::APP_ISSUES_URL)
        .support_url(config::APP_SUPPORT_URL)
        .developer_name(config::APP_AUTHOR_NAME)
        .developers(config::APP_AUTHORS.lines().collect::<Vec<&str>>())
        // TRANSLATORS: 'Name <email@domain.com>' or 'Name https://website.example'
        // .translator_credits(gettext("translator-credits"))
        .license_type(globals::APP_LICENSE)
        .version(
            (String::from(config::APP_VERSION) + config::APP_VERSION_SUFFIX).as_str(),
        )
        .build();

    if config::PROFILE == "devel" {
        aboutdialog.add_css_class("devel");
    }

    aboutdialog.present(appwindow.root().as_ref());
}

pub(crate) fn dialog_keyboard_shortcuts(appwindow: &RnAppWindow) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/shortcuts.ui").as_str(),
    );
    let dialog: ShortcutsWindow = builder.object("shortcuts_window").unwrap();
    dialog.set_transient_for(Some(appwindow));
    dialog.present();
}

/// Only to be called from the tabview close-page handler
///
/// Returns `close_finish_confirm` that should be passed into close_page_finish() and indicates if the tab should be
/// actually closed or closing should be aborted.
// #[must_use]
#[allow(unused)]
pub(crate) async fn dialog_close_tab(
    appwindow: &RnAppWindow,
    tab_page: &adw::TabPage,
) -> bool {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: adw::AlertDialog = builder.object("dialog_close_tab").unwrap();
    let file_group: adw::PreferencesGroup =
        builder.object("close_tab_file_group").unwrap();
    // Returns close_finish_confirm, a boolean that indicates if the tab should actually be closed or closing
    // should be aborted.
    match dialog.choose_future(appwindow).await.as_str() {
        "discard" => true,
        "save" => true,
        _ => {
            // Cancel
            false
        }
    }
}
#[allow(unused)]
pub(crate) async fn dialog_close_window(appwindow: &RnAppWindow) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: adw::AlertDialog = builder.object("dialog_close_window").unwrap();
    let files_group: adw::PreferencesGroup =
        builder.object("close_window_files_group").unwrap();

    let close = match dialog.choose_future(appwindow).await.as_str() {
        "discard" => {
            // do nothing and close
            true
        }
        "save" => {
            // appwindow.overlays().progressbar_finish();
            true
        }
        _ => {
            // Cancel
            false
        }
    };

    if close {
        appwindow.close_force();
    }
}
#[allow(unused)]
pub(crate) async fn dialog_trash_file(
    appwindow: &RnAppWindow,
    current_file: &gio::File,
) {
    let builder = Builder::from_resource(
        (String::from(config::APP_IDPATH) + "ui/dialogs/dialogs.ui").as_str(),
    );
    let dialog: adw::AlertDialog = builder.object("dialog_trash_file").unwrap();

    match dialog.choose_future(appwindow).await.as_str() {
        "trash" => {
            glib::spawn_future_local(clone!(
                #[weak]
                appwindow,
                #[strong]
                current_file,
                async move {
                    current_file.trash_async(
                        glib::source::Priority::DEFAULT,
                        None::<&gio::Cancellable>,
                        clone!(
                            #[weak]
                            appwindow,
                            #[strong]
                            current_file,
                            move |res| {
                                if let Err(e) = res {
                                    // appwindow
                                    //     .overlays()
                                    //     .dispatch_toast_error(&gettext("Trashing file failed"));
                                    error!(
                                        "Trash filerow file `{current_file:?}` failed , Err: {e:?}"
                                    );
                                    return;
                                }
                            }
                        ),
                    );
                }
            ));
        }
        _ => {
            // Cancel
        }
    }
}
#[allow(unused)]
const WORKSPACELISTENTRY_ICONS_LIST: &[&str] = &[
    "workspacelistentryicon-bandaid-symbolic",
    "workspacelistentryicon-bank-symbolic",
    "workspacelistentryicon-bookmark-symbolic",
    "workspacelistentryicon-book-symbolic",
    "workspacelistentryicon-bread-symbolic",
    "workspacelistentryicon-calendar-symbolic",
    "workspacelistentryicon-camera-symbolic",
    "workspacelistentryicon-chip-symbolic",
    "workspacelistentryicon-clock-symbolic",
    "workspacelistentryicon-code-symbolic",
    "workspacelistentryicon-compose-symbolic",
    "workspacelistentryicon-crop-symbolic",
    "workspacelistentryicon-dictionary-symbolic",
    "workspacelistentryicon-document-symbolic",
    "workspacelistentryicon-drinks-symbolic",
    "workspacelistentryicon-flag-symbolic",
    "workspacelistentryicon-folder-symbolic",
    "workspacelistentryicon-footprints-symbolic",
    "workspacelistentryicon-gamepad-symbolic",
    "workspacelistentryicon-gear-symbolic",
    "workspacelistentryicon-globe-symbolic",
    "workspacelistentryicon-hammer-symbolic",
    "workspacelistentryicon-heart-symbolic",
    "workspacelistentryicon-hourglass-symbolic",
    "workspacelistentryicon-key-symbolic",
    "workspacelistentryicon-language-symbolic",
    "workspacelistentryicon-library-symbolic",
    "workspacelistentryicon-lightbulb-symbolic",
    "workspacelistentryicon-math-symbolic",
    "workspacelistentryicon-meeting-symbolic",
    "workspacelistentryicon-money-symbolic",
    "workspacelistentryicon-musicnote-symbolic",
    "workspacelistentryicon-nature-symbolic",
    "workspacelistentryicon-open-book-symbolic",
    "workspacelistentryicon-paintbrush-symbolic",
    "workspacelistentryicon-pencilandpaper-symbolic",
    "workspacelistentryicon-people-symbolic",
    "workspacelistentryicon-person-symbolic",
    "workspacelistentryicon-projector-symbolic",
    "workspacelistentryicon-science-symbolic",
    "workspacelistentryicon-scratchpad-symbolic",
    "workspacelistentryicon-shapes-symbolic",
    "workspacelistentryicon-shopping-symbolic",
    "workspacelistentryicon-speechbubble-symbolic",
    "workspacelistentryicon-speedometer-symbolic",
    "workspacelistentryicon-star-symbolic",
    "workspacelistentryicon-terminal-symbolic",
    "workspacelistentryicon-text-symbolic",
    "workspacelistentryicon-travel-symbolic",
    "workspacelistentryicon-weather-symbolic",
    "workspacelistentryicon-weight-symbolic",
];
