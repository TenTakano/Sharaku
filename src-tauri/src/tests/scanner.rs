use super::*;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn recognizes_supported_extensions() {
    for ext in &["jpg", "jpeg", "png", "gif", "webp", "bmp"] {
        let name = format!("photo.{}", ext);
        assert!(is_image_file(Path::new(&name)), "should accept .{}", ext);
    }
}

#[test]
fn recognizes_uppercase_and_mixed_case() {
    assert!(is_image_file(Path::new("photo.JPG")));
    assert!(is_image_file(Path::new("photo.Png")));
    assert!(is_image_file(Path::new("photo.GIF")));
    assert!(is_image_file(Path::new("photo.WeBp")));
}

#[test]
fn rejects_non_image_extensions() {
    for ext in &["pdf", "zip", "txt", "mp4", "doc", "rs"] {
        let name = format!("file.{}", ext);
        assert!(!is_image_file(Path::new(&name)), "should reject .{}", ext);
    }
}

#[test]
fn rejects_no_extension() {
    assert!(!is_image_file(Path::new("README")));
    assert!(!is_image_file(Path::new("Makefile")));
}

#[test]
fn rejects_hidden_files_without_image_ext() {
    assert!(!is_image_file(Path::new(".gitignore")));
    assert!(!is_image_file(Path::new(".hidden")));
}

#[test]
fn accepts_hidden_files_with_image_ext() {
    assert!(is_image_file(Path::new(".photo.jpg")));
}

#[test]
fn has_image_subfolders_returns_true_with_images_in_subdirectory() {
    let tmp = TempDir::new().unwrap();
    let sub = tmp.path().join("sub");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("photo.jpg"), b"fake").unwrap();

    assert!(has_image_subfolders(tmp.path()));
}

#[test]
fn has_image_subfolders_returns_false_for_empty_subdirectories() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir(tmp.path().join("sub")).unwrap();

    assert!(!has_image_subfolders(tmp.path()));
}

#[test]
fn has_image_subfolders_ignores_images_in_root() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("photo.jpg"), b"fake").unwrap();

    assert!(!has_image_subfolders(tmp.path()));
}

#[test]
fn has_image_subfolders_detects_deeply_nested_images() {
    let tmp = TempDir::new().unwrap();
    let deep = tmp.path().join("a").join("b").join("c");
    fs::create_dir_all(&deep).unwrap();
    fs::write(deep.join("image.png"), b"fake").unwrap();

    assert!(has_image_subfolders(tmp.path()));
}

#[test]
fn content_type_jpeg() {
    assert_eq!(content_type_from_path("/path/to/image.jpg"), "image/jpeg");
    assert_eq!(content_type_from_path("/path/to/image.JPEG"), "image/jpeg");
}

#[test]
fn content_type_png() {
    assert_eq!(content_type_from_path("/path/to/image.png"), "image/png");
}

#[test]
fn content_type_gif() {
    assert_eq!(content_type_from_path("/path/to/image.gif"), "image/gif");
}

#[test]
fn content_type_webp() {
    assert_eq!(content_type_from_path("/path/to/image.webp"), "image/webp");
}

#[test]
fn content_type_bmp() {
    assert_eq!(content_type_from_path("/path/to/image.bmp"), "image/bmp");
}

#[test]
fn content_type_unknown() {
    assert_eq!(
        content_type_from_path("/path/to/file.xyz"),
        "application/octet-stream"
    );
}
