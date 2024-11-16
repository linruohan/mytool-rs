#/usr/bin/env bash
if uname -s | grep MINGW; then
    # window
    glib_schemas_dir="/c/ProgramData/glib-2.0/schemas/"
else
    # linux
    glib_schemas_dir="$HOME/.local/share/glib-2.0/schemas"
fi
mkdir -p "${glib_schemas_dir}"
cp com.github.linruohan.mytool-rs.gschema.xml "${glib_schemas_dir}"
glib-compile-schemas "${glib_schemas_dir}"
echo "glib-compile-schemas ${glib_schemas_dir} successfully!"
