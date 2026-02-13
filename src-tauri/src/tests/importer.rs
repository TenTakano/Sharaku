use std::path::Path;

use super::*;

// parse_folder_name tests

#[test]
fn parse_bracket_pattern() {
    let result = parse_folder_name("[Artist Name] Work Title");
    assert_eq!(result.title, "Work Title");
    assert_eq!(result.artist.as_deref(), Some("Artist Name"));
}

#[test]
fn parse_dash_pattern() {
    let result = parse_folder_name("Artist Name - Work Title");
    assert_eq!(result.title, "Work Title");
    assert_eq!(result.artist.as_deref(), Some("Artist Name"));
}

#[test]
fn parse_plain_name() {
    let result = parse_folder_name("Just A Title");
    assert_eq!(result.title, "Just A Title");
    assert_eq!(result.artist, None);
}

#[test]
fn parse_bracket_empty_artist() {
    let result = parse_folder_name("[] Title");
    assert_eq!(result.title, "[] Title");
    assert_eq!(result.artist, None);
}

#[test]
fn parse_bracket_empty_title() {
    let result = parse_folder_name("[Artist]");
    assert_eq!(result.title, "[Artist]");
    assert_eq!(result.artist, None);
}

#[test]
fn parse_dash_with_no_spaces() {
    let result = parse_folder_name("no-dash-pattern");
    assert_eq!(result.title, "no-dash-pattern");
    assert_eq!(result.artist, None);
}

#[test]
fn parse_japanese_bracket() {
    let result = parse_folder_name("[サークル名] 作品タイトル");
    assert_eq!(result.title, "作品タイトル");
    assert_eq!(result.artist.as_deref(), Some("サークル名"));
}

#[test]
fn parse_japanese_dash() {
    let result = parse_folder_name("アーティスト - 作品名");
    assert_eq!(result.title, "作品名");
    assert_eq!(result.artist.as_deref(), Some("アーティスト"));
}

// list_images_in_folder tests

#[test]
fn list_images_finds_image_files() {
    let dir = std::env::temp_dir().join("sharaku_test_list_images");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    std::fs::write(dir.join("01.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("02.png"), b"fake").unwrap();
    std::fs::write(dir.join("readme.txt"), b"text").unwrap();

    let images = list_images_in_folder(&dir).unwrap();
    assert_eq!(images.len(), 2);
    assert!(
        images[0].file_name().unwrap().to_str().unwrap()
            <= images[1].file_name().unwrap().to_str().unwrap()
    );

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn list_images_empty_folder() {
    let dir = std::env::temp_dir().join("sharaku_test_list_images_empty");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let images = list_images_in_folder(&dir).unwrap();
    assert!(images.is_empty());

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn list_images_sorted_order() {
    let dir = std::env::temp_dir().join("sharaku_test_list_sorted");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    std::fs::write(dir.join("c.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("a.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("b.jpg"), b"fake").unwrap();

    let images = list_images_in_folder(&dir).unwrap();
    assert_eq!(images.len(), 3);
    assert_eq!(images[0].file_name().unwrap(), "a.jpg");
    assert_eq!(images[1].file_name().unwrap(), "b.jpg");
    assert_eq!(images[2].file_name().unwrap(), "c.jpg");

    std::fs::remove_dir_all(&dir).unwrap();
}

// preview_import_path tests

#[test]
fn preview_path_with_template() {
    let metadata = WorkMetadata {
        title: "My Work".to_string(),
        artist: Some("Artist".to_string()),
        year: None,
        genre: None,
        circle: None,
        origin: None,
    };
    let result = preview_import_path(Path::new("/library"), "{artist}/{title}", &metadata);
    assert_eq!(result, "/library/Artist/My Work");
}
