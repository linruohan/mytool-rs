fn main() {
    // actions
    glib_build_tools::compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        "mytool.gresource",
    );
}
