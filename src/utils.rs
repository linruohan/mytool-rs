// ANCHOR_END: data_path
// Imports
use crate::config::APP_ID;
use anyhow::Context;
use futures::AsyncWriteExt;
use gtk::glib;
use gtk::{gdk, gio, prelude::*};
use path_absolutize::Absolutize;
use std::cell::Ref;
use std::path::{Path, PathBuf};
use std::slice::Iter;

// ANCHOR: data_path
#[allow(unused)]
pub fn data_path() -> PathBuf {
    let mut path = glib::user_data_dir();
    path.push(APP_ID);
    std::fs::create_dir_all(&path).expect("Could not create directory.");
    path.push("data.json");
    path
}
#[allow(unused)]
/// Create a new file or replace if it already exists, asynchronously.
pub(crate) async fn create_replace_file_future(
    bytes: Vec<u8>,
    file: &gio::File,
) -> anyhow::Result<()> {
    let Some(file_path) = file.path() else {
        return Err(anyhow::anyhow!(
            "Can't create-replace file that has no path."
        ));
    };
    let mut write_file = async_fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&file_path)
        .await
        .context(format!(
            "Failed to create/open/truncate file for path '{}'",
            file_path.display()
        ))?;
    write_file.write_all(&bytes).await.context(format!(
        "Failed to write bytes to file with path '{}'",
        file_path.display()
    ))?;
    write_file.sync_all().await.context(format!(
        "Failed to sync file after writing with path '{}'",
        file_path.display()
    ))?;
    Ok(())
}

#[allow(unused)]
pub(crate) fn str_from_u8_nul_utf8(
    utf8_src: &[u8],
) -> Result<&str, std::str::Utf8Error> {
    let nul_range_end = utf8_src
        .iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len()); // default to length if no `\0` present
    std::str::from_utf8(&utf8_src[0..nul_range_end])
}
#[allow(unused)]
/// Get the index of the AxisUse enum
///
/// TODO: Report to gtk-rs that [gdk::AxisUse] needs a [`Into<std::ops::Index>`] implementation
/// for usage to retrieve pointer axes in [gdk::TimeCoord]
pub(crate) fn axis_use_idx(a: gdk::AxisUse) -> usize {
    match a {
        gdk::AxisUse::Ignore => 0,
        gdk::AxisUse::X => 1,
        gdk::AxisUse::Y => 2,
        gdk::AxisUse::DeltaX => 3,
        gdk::AxisUse::DeltaY => 4,
        gdk::AxisUse::Pressure => 5,
        gdk::AxisUse::Xtilt => 6,
        gdk::AxisUse::Ytilt => 7,
        gdk::AxisUse::Wheel => 8,
        gdk::AxisUse::Distance => 9,
        gdk::AxisUse::Rotation => 10,
        gdk::AxisUse::Slider => 11,
        _ => unreachable!(),
    }
}
#[allow(unused)]
pub fn now_formatted_string() -> String {
    chrono::Local::now().format("%Y-%m-%d_%H:%M:%S").to_string()
}
#[allow(unused)]
pub(crate) fn default_file_title_for_export(
    output_file: Option<gio::File>,
    fallback: Option<&str>,
    suffix: Option<&str>,
) -> String {
    let mut title = output_file
        .and_then(|f| Some(f.basename()?.file_stem()?.to_string_lossy().to_string()))
        .unwrap_or_else(|| {
            fallback
                .map(|f| f.to_owned())
                .unwrap_or_else(now_formatted_string)
        });

    if let Some(suffix) = suffix {
        title += suffix;
    }

    title
}
#[allow(unused)]
/// Absolutizes two paths and checks if they are equal.
///
/// Compared to `canonicalize()`, the files do not need to exist on the fs and symlinks are not resolved.
#[inline]
pub(crate) fn paths_abs_eq(
    first: impl AsRef<Path>,
    second: impl AsRef<Path>,
) -> anyhow::Result<bool> {
    let first = first.as_ref().absolutize()?.into_owned();
    let second = second.as_ref().absolutize()?.into_owned();
    Ok(first == second)
}

/// Wrapper type that enables iterating over [`std::cell::RefCell<Vec<T>>`]
pub(crate) struct VecRefWrapper<'a, T: 'a> {
    r: Ref<'a, Vec<T>>,
}

impl<'a, 'b: 'a, T: 'a> IntoIterator for &'b VecRefWrapper<'a, T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Iter<'a, T> {
        self.r.iter()
    }
}

impl<'a, T> VecRefWrapper<'a, T>
where
    T: 'a,
{
    #[allow(unused)]
    pub(crate) fn new(r: Ref<'a, Vec<T>>) -> Self {
        Self { r }
    }
}
#[allow(unused)]
pub(crate) fn path_walk_up_until_exists(
    path: impl AsRef<Path>,
) -> anyhow::Result<PathBuf> {
    let mut path = path.as_ref().absolutize()?.to_path_buf();
    while !path.exists() {
        path = path
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| anyhow::anyhow!("Path {} has no parent", path.display()))?;
    }
    Ok(path)
}
