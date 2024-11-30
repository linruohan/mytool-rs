# config.rs
message('Configuring config.rs file')

global_conf = configuration_data()
global_conf.set_quoted('APP_NAME', app_name)
global_conf.set_quoted('APP_NAME_CAPITALIZED', app_name_capitalized)
global_conf.set_quoted('APP_ID', app_id)
global_conf.set_quoted('APP_IDPATH', app_idpath)
global_conf.set_quoted('APP_VERSION', version)
global_conf.set_quoted('APP_VERSION_SUFFIX', version_suffix)
global_conf.set_quoted('APP_AUTHOR_NAME', app_author_name)
global_conf.set_quoted('APP_AUTHORS', app_authors)
global_conf.set_quoted('APP_WEBSITE', app_website)
global_conf.set_quoted('APP_ISSUES_URL', app_issues_url)
global_conf.set_quoted('APP_SUPPORT_URL', app_support_url)
global_conf.set_quoted('APP_DONATE_URL', app_donate_url)
global_conf.set_quoted('PROFILE', profile)
global_conf.set_quoted('GETTEXT_PACKAGE', gettext_package)
global_conf.set_quoted('DATADIR', datadir)
global_conf.set_quoted('LIBDIR', libdir)
global_conf.set_quoted('LOCALEDIR', localedir)

configure_file(
    input: 'config.rs.in',
    output: 'config.rs',
    configuration: global_conf,
)

# Copy the config.rs output to the source directory
run_command(
    'cp',
    meson.project_build_root() /  'src' / 'config.rs',
    meson.project_source_root() / 'src' / 'config.rs',
    check: true,
)

mytool_ui_sources = files(
    'app/appactions.rs',
    'app/mod.rs',
    'appwindow/actions.rs',
    'appwindow/appsettings.rs',
    'appwindow/imp.rs',
    'appwindow/mod.rs',
    'canvas/mod.rs',
    'collection_object/imp.rs',
    'collection_object/mod.rs',
    'task_object/imp.rs',
    'task_object/mod.rs',
    'dialogs/mod.rs',
    'appmenu.rs',
    'canvaswrapper.rs',
    'config.rs',
    'env.rs',
    'filetype.rs',
    'globals.rs',
    'main.rs',
    'mainheader.rs',
    'overlays.rs',
    'sidebar.rs',
    'utils.rs',
)
