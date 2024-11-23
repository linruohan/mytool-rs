#![warn(missing_debug_implementations)]
#![allow(clippy::single_match)]
// Turns off console window on Windows, but not when building with dev profile.
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

pub(crate) mod app;
pub(crate) mod appmenu;
pub(crate) mod appwindow;
// pub(crate) mod canvas;
pub(crate) mod canvaswrapper;
mod collection_object;
pub(crate) mod config;
// pub(crate) mod contextmenu;
pub(crate) mod dialogs;
pub(crate) mod env;
pub(crate) mod filetype;
pub(crate) mod globals;
pub(crate) mod mainheader;
pub(crate) mod overlays;
pub(crate) mod sidebar;
mod task_object;
mod utils;
mod window;

pub(crate) use app::RnApp;
pub(crate) use appmenu::RnAppMenu;
pub(crate) use appwindow::RnAppWindow;
// pub(crate) use canvas::RnCanvas;
pub(crate) use canvaswrapper::RnCanvasWrapper;
// pub(crate) use contextmenu::RnContextMenu;
pub(crate) use filetype::FileType;
pub(crate) use mainheader::RnMainHeader;
pub(crate) use overlays::RnOverlays;
pub(crate) use sidebar::RnSidebar;
use window::Window;

// Renames
extern crate nalgebra as na;
// Imports
use adw::prelude::*;
use anyhow::Context;
use gtk::{gio, glib};
use tracing::debug;
fn main() -> glib::ExitCode {
    if let Err(e) = setup_tracing() {
        eprintln!("failed to setup tracing, Err: {e:?}");
    }
    if let Err(e) = env::setup_env() {
        eprintln!("failed to setup env, Err: {e:?}");
    }
    if let Err(e) = setup_i18n() {
        eprintln!("failed to setup i18n, Err: {e:?}");
    }
    if let Err(e) = setup_gresources() {
        eprintln!("failed to setup gresources, Err: {e:?}");
    }
    // // Create a new application
    // let app = adw::Application::builder()
    //     .application_id(config::APP_ID)
    //     .build();

    // // Connect to signals
    // app.connect_startup(setup_shortcuts);
    // app.connect_activate(build_ui);

    // // Run the application
    // app.run()
    let app = RnApp::new();
    app.run()
}
#[allow(unused)]
fn setup_shortcuts(app: &adw::Application) {
    app.set_accels_for_action("win.filter('All')", &["<Ctrl>a"]);
    app.set_accels_for_action("win.filter('Open')", &["<Ctrl>o"]);
    app.set_accels_for_action("win.filter('Done')", &["<Ctrl>d"]);
}
#[allow(unused)]
fn build_ui(app: &adw::Application) {
    // Create a new custom window and present it
    let window = Window::new(app);
    window.present();
}

fn setup_tracing() -> anyhow::Result<()> {
    let timer = tracing_subscriber::fmt::time::Uptime::default();

    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_timer(timer)
        .try_init()
        .map_err(|e| anyhow::anyhow!(e))?;
    debug!(".. tracing subscriber initialized.");
    Ok(())
}

fn setup_i18n() -> anyhow::Result<()> {
    let _locale_dir = env::locale_dir()?;

    // gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, "");
    // gettextrs::bindtextdomain(config::GETTEXT_PACKAGE, locale_dir)?;
    // gettextrs::bind_textdomain_codeset(config::GETTEXT_PACKAGE, "UTF-8")?;
    // gettextrs::textdomain(config::GETTEXT_PACKAGE)?;
    Ok(())
}

fn setup_gresources() -> anyhow::Result<()> {
    gio::resources_register_include!("mytool.gresource")
        .context("Failed to register and include compiled gresource.")
}
