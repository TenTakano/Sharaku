use std::path::Path;
use walkdir::WalkDir;

const IMAGE_EXTENSION_MIME_TYPES: &[(&str, &str)] = &[
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("png", "image/png"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("bmp", "image/bmp"),
];

pub(crate) fn is_image_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            let ext = ext.to_ascii_lowercase();
            IMAGE_EXTENSION_MIME_TYPES
                .iter()
                .any(|(known_ext, _)| *known_ext == ext)
        })
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

pub(crate) fn content_type_from_path(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    IMAGE_EXTENSION_MIME_TYPES
        .iter()
        .find(|(known_ext, _)| *known_ext == ext)
        .map(|(_, mime)| *mime)
        .unwrap_or("application/octet-stream")
}

#[cfg(test)]
#[path = "tests/scanner.rs"]
mod tests;
