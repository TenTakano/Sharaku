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

// paths_overlap tests

#[test]
fn paths_overlap_identical() {
    assert!(paths_overlap(Path::new("/a/b"), Path::new("/a/b")));
}

#[test]
fn paths_overlap_source_contains_dest() {
    assert!(paths_overlap(Path::new("/a"), Path::new("/a/b")));
}

#[test]
fn paths_overlap_dest_contains_source() {
    assert!(paths_overlap(Path::new("/a/b/c"), Path::new("/a/b")));
}

#[test]
fn paths_overlap_disjoint() {
    assert!(!paths_overlap(Path::new("/a/b"), Path::new("/c/d")));
}

#[test]
fn paths_overlap_partial_name_no_overlap() {
    assert!(!paths_overlap(
        Path::new("/library/art"),
        Path::new("/library/artist")
    ));
}

// Natural sort tests

#[test]
fn list_images_natural_sort_order() {
    let dir = std::env::temp_dir().join("sharaku_test_natord");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    std::fs::write(dir.join("page1.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("page2.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("page10.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("page20.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("page3.jpg"), b"fake").unwrap();

    let images = list_images_in_folder(&dir).unwrap();
    assert_eq!(images.len(), 5);
    assert_eq!(images[0].file_name().unwrap(), "page1.jpg");
    assert_eq!(images[1].file_name().unwrap(), "page2.jpg");
    assert_eq!(images[2].file_name().unwrap(), "page3.jpg");
    assert_eq!(images[3].file_name().unwrap(), "page10.jpg");
    assert_eq!(images[4].file_name().unwrap(), "page20.jpg");

    std::fs::remove_dir_all(&dir).unwrap();
}

// count_direct_images tests

#[test]
fn count_direct_images_only_counts_immediate() {
    let dir = std::env::temp_dir().join("sharaku_test_count_direct");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    std::fs::write(dir.join("a.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("b.png"), b"fake").unwrap();
    std::fs::write(dir.join("readme.txt"), b"text").unwrap();

    let sub = dir.join("subdir");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(sub.join("c.jpg"), b"fake").unwrap();

    assert_eq!(count_direct_images(&dir), 2);

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn count_direct_images_empty_dir() {
    let dir = std::env::temp_dir().join("sharaku_test_count_empty");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    assert_eq!(count_direct_images(&dir), 0);

    std::fs::remove_dir_all(&dir).unwrap();
}
