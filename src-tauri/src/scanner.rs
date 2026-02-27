use std::path::Path;
use walkdir::WalkDir;

pub(crate) const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp"];

pub(crate) fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| IMAGE_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

pub(crate) fn count_direct_images(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_type().map(|ft| ft.is_file()).unwrap_or(false)
                        && is_image_file(&e.path())
                })
                .count()
        })
        .unwrap_or(0)
}

pub(crate) fn has_image_subfolders(dir: &Path) -> bool {
    WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .any(|e| count_direct_images(e.path()) > 0)
}

#[cfg(test)]
#[path = "tests/scanner.rs"]
mod tests;
