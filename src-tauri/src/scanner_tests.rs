use super::*;
use std::path::Path;

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
