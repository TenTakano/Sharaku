use std::path::Path;

use super::*;

fn full_metadata() -> WorkMetadata {
    WorkMetadata {
        title: "My Title".to_string(),
        artist: Some("Artist A".to_string()),
        year: Some(2024),
        genre: Some("Manga".to_string()),
        circle: Some("Circle X".to_string()),
        origin: Some("Original".to_string()),
    }
}

fn partial_metadata() -> WorkMetadata {
    WorkMetadata {
        title: "My Title".to_string(),
        artist: None,
        year: None,
        genre: None,
        circle: None,
        origin: None,
    }
}

// validate_template tests

#[test]
fn validate_valid_template() {
    assert!(validate_template("{title}").is_ok());
    assert!(validate_template("{artist}/{title}").is_ok());
    assert!(validate_template("{year}/{genre}/{title}").is_ok());
    assert!(validate_template("prefix-{title}-suffix").is_ok());
}

#[test]
fn validate_missing_title() {
    let err = validate_template("{artist}/{genre}").unwrap_err();
    assert!(err.to_string().contains("{title}"));
}

#[test]
fn validate_unknown_placeholder() {
    let err = validate_template("{title}/{unknown}").unwrap_err();
    assert!(err.to_string().contains("未知のプレースホルダー"));
}

#[test]
fn validate_empty_template() {
    let err = validate_template("").unwrap_err();
    assert!(err.to_string().contains("空"));
}

#[test]
fn validate_whitespace_only() {
    let err = validate_template("   ").unwrap_err();
    assert!(err.to_string().contains("空"));
}

#[test]
fn validate_unclosed_placeholder() {
    let err = validate_template("{title").unwrap_err();
    assert!(err.to_string().contains("閉じられていない"));
}

#[test]
fn validate_empty_placeholder() {
    let err = validate_template("{}/{title}").unwrap_err();
    assert!(err.to_string().contains("空のプレースホルダー"));
}

// render_template tests

#[test]
fn render_all_fields() {
    let result = render_template("{artist}/{title}", &full_metadata());
    assert_eq!(result, "Artist A/My Title");
}

#[test]
fn render_with_none_fields() {
    let result = render_template("{artist}/{title}", &partial_metadata());
    assert_eq!(result, "Unknown/My Title");
}

#[test]
fn render_year_numeric() {
    let result = render_template("{year}/{title}", &full_metadata());
    assert_eq!(result, "2024/My Title");
}

#[test]
fn render_year_none() {
    let result = render_template("{year}/{title}", &partial_metadata());
    assert_eq!(result, "Unknown/My Title");
}

#[test]
fn render_sanitizes_forbidden_chars() {
    let meta = WorkMetadata {
        title: "My:Title*With?Bad<Chars>".to_string(),
        artist: Some("Art\\ist|Name\"Test".to_string()),
        year: None,
        genre: None,
        circle: None,
        origin: None,
    };
    let result = render_template("{artist}/{title}", &meta);
    assert!(!result.contains(':'));
    assert!(!result.contains('*'));
    assert!(!result.contains('?'));
    assert!(!result.contains('<'));
    assert!(!result.contains('>'));
    assert!(!result.contains('\\'));
    assert!(!result.contains('|'));
    assert!(!result.contains('"'));
}

#[test]
fn render_all_placeholders() {
    let result = render_template(
        "{artist}/{circle}/{genre}/{origin}/{year}/{title}",
        &full_metadata(),
    );
    assert_eq!(result, "Artist A/Circle X/Manga/Original/2024/My Title");
}

#[test]
fn render_with_literal_text() {
    let result = render_template("works/{artist} - {title}", &full_metadata());
    assert_eq!(result, "works/Artist A - My Title");
}

// resolve_work_path tests

#[test]
fn resolve_path_simple() {
    let root = Path::new("/library");
    let path = resolve_work_path(root, "{title}", &full_metadata());
    assert_eq!(path, Path::new("/library/My Title"));
}

#[test]
fn resolve_path_nested() {
    let root = Path::new("/library");
    let path = resolve_work_path(root, "{artist}/{year}/{title}", &full_metadata());
    assert_eq!(path, Path::new("/library/Artist A/2024/My Title"));
}

// resolve_unique_work_path tests

#[test]
fn resolve_unique_nonexistent_returns_base() {
    let dir = std::env::temp_dir().join("sharaku_test_unique_nonexist");
    let _ = std::fs::remove_dir_all(&dir);

    let path = resolve_unique_work_path(&dir, "{title}", &full_metadata());
    assert_eq!(path, dir.join("My Title"));
}

#[test]
fn resolve_unique_existing_gets_suffix() {
    let dir = std::env::temp_dir().join("sharaku_test_unique_exist");
    let _ = std::fs::remove_dir_all(&dir);
    let target = dir.join("My Title");
    std::fs::create_dir_all(&target).unwrap();

    let path = resolve_unique_work_path(&dir, "{title}", &full_metadata());
    assert_ne!(path, target);
    assert!(path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("My Title_"));

    std::fs::remove_dir_all(&dir).unwrap();
}
