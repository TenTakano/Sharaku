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
        work_type: None,
    };
    let result = preview_import_path(
        Path::new("/library"),
        "{artist}/{title}",
        &metadata,
        crate::template::WORK_KIND_FOLDER,
    );
    assert_eq!(result, "/library/works/Artist/My Work");
}

#[test]
fn preview_path_for_image_kind() {
    let metadata = WorkMetadata {
        title: "Sketch".to_string(),
        artist: Some("Artist".to_string()),
        year: None,
        genre: None,
        circle: None,
        origin: None,
        work_type: None,
    };
    let result = preview_import_path(
        Path::new("/library"),
        "{artist}/{title}",
        &metadata,
        crate::template::WORK_KIND_IMAGE,
    );
    assert_eq!(result, "/library/pictures/Artist/Sketch");
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

// find_leaf_indices tests

#[test]
fn leaf_indices_flat_structure() {
    let candidates = vec![
        (PathBuf::from("/root/work1"), 3),
        (PathBuf::from("/root/work2"), 5),
    ];
    let leaves = find_leaf_indices(&candidates, &[Path::new("/root")]);
    assert_eq!(leaves.len(), 2);
    assert!(leaves.contains(&0));
    assert!(leaves.contains(&1));
}

#[test]
fn leaf_indices_nested_skips_intermediate() {
    let candidates = vec![
        (PathBuf::from("/root"), 1),
        (PathBuf::from("/root/chapter1"), 3),
        (PathBuf::from("/root/chapter2"), 5),
    ];
    let leaves = find_leaf_indices(&candidates, &[Path::new("/root")]);
    assert_eq!(leaves.len(), 2);
    assert!(leaves.contains(&1));
    assert!(leaves.contains(&2));
    assert!(!leaves.contains(&0));
}

#[test]
fn leaf_indices_deeply_nested() {
    let candidates = vec![
        (PathBuf::from("/root"), 1),
        (PathBuf::from("/root/level1"), 2),
        (PathBuf::from("/root/level1/level2"), 3),
    ];
    let leaves = find_leaf_indices(&candidates, &[Path::new("/root")]);
    assert_eq!(leaves.len(), 1);
    assert!(leaves.contains(&2));
}

#[test]
fn leaf_indices_single_leaf() {
    let candidates = vec![(PathBuf::from("/root"), 5)];
    let leaves = find_leaf_indices(&candidates, &[Path::new("/root")]);
    assert_eq!(leaves.len(), 1);
    assert!(leaves.contains(&0));
}

#[test]
fn leaf_indices_mixed_branches() {
    // root/
    //   cover.jpg         <- intermediate (has child with images)
    //   branch_a/
    //     page.jpg         <- leaf
    //   branch_b/
    //     page.jpg         <- intermediate
    //     sub/
    //       page.jpg       <- leaf
    let candidates = vec![
        (PathBuf::from("/root"), 1),
        (PathBuf::from("/root/branch_a"), 1),
        (PathBuf::from("/root/branch_b"), 1),
        (PathBuf::from("/root/branch_b/sub"), 1),
    ];
    let leaves = find_leaf_indices(&candidates, &[Path::new("/root")]);
    assert_eq!(leaves.len(), 2);
    assert!(leaves.contains(&1)); // branch_a
    assert!(leaves.contains(&3)); // branch_b/sub
}

#[test]
fn leaf_indices_multiple_independent_roots() {
    let candidates = vec![
        (PathBuf::from("/root_a"), 1),
        (PathBuf::from("/root_a/child"), 2),
        (PathBuf::from("/root_b/work1"), 3),
        (PathBuf::from("/root_b/work2"), 4),
    ];
    let leaves = find_leaf_indices(&candidates, &[Path::new("/root_a"), Path::new("/root_b")]);
    assert_eq!(leaves.len(), 3);
    assert!(leaves.contains(&1)); // root_a/child
    assert!(leaves.contains(&2)); // root_b/work1
    assert!(leaves.contains(&3)); // root_b/work2
    assert!(!leaves.contains(&0)); // root_a is intermediate
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

    assert_eq!(crate::scanner::count_direct_images(&dir), 2);

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn count_direct_images_empty_dir() {
    let dir = std::env::temp_dir().join("sharaku_test_count_empty");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    assert_eq!(crate::scanner::count_direct_images(&dir), 0);

    std::fs::remove_dir_all(&dir).unwrap();
}

// parse_image_file_name tests

#[test]
fn parse_image_file_name_strips_extension() {
    let result = parse_image_file_name("[Artist] Work.jpg");
    assert_eq!(result.title, "Work");
    assert_eq!(result.artist.as_deref(), Some("Artist"));
}

#[test]
fn parse_image_file_name_no_extension() {
    let result = parse_image_file_name("plain_name");
    assert_eq!(result.title, "plain_name");
    assert_eq!(result.artist, None);
}

#[test]
fn parse_image_file_name_multi_dot() {
    let result = parse_image_file_name("Artist - Title.v2.png");
    assert_eq!(result.title, "Title.v2");
    assert_eq!(result.artist.as_deref(), Some("Artist"));
}

// import_single_image error cases

#[test]
fn import_single_image_rejects_non_image_file() {
    let dir = std::env::temp_dir().join("sharaku_test_import_single_non_image");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let txt = dir.join("note.txt");
    std::fs::write(&txt, b"text").unwrap();

    let conn = crate::db::open_db_in_memory().unwrap();
    crate::library::add_library(&conn, "Test", Some("/tmp")).ok();

    let request = ImportRequest {
        source_path: txt.to_string_lossy().to_string(),
        title: "x".into(),
        artist: None,
        year: None,
        genre: None,
        circle: None,
        origin: None,
        mode: ImportMode::Copy,
        kind: ImportKind::Image,
    };

    let lib_id = {
        let mut stmt = conn.prepare("SELECT id FROM libraries LIMIT 1").unwrap();
        stmt.query_row([], |row| row.get::<_, String>(0)).unwrap()
    };

    let result = import_single_image(&request, &conn, &lib_id, &dir);
    assert!(result.is_err());

    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn import_single_image_rejects_directory_path() {
    let dir = std::env::temp_dir().join("sharaku_test_import_single_dir_reject");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let conn = crate::db::open_db_in_memory().unwrap();
    crate::library::add_library(&conn, "Test", Some("/tmp")).ok();

    let request = ImportRequest {
        source_path: dir.to_string_lossy().to_string(),
        title: "x".into(),
        artist: None,
        year: None,
        genre: None,
        circle: None,
        origin: None,
        mode: ImportMode::Copy,
        kind: ImportKind::Image,
    };

    let lib_id = {
        let mut stmt = conn.prepare("SELECT id FROM libraries LIMIT 1").unwrap();
        stmt.query_row([], |row| row.get::<_, String>(0)).unwrap()
    };

    let result = import_single_image(&request, &conn, &lib_id, &dir);
    assert!(result.is_err());

    std::fs::remove_dir_all(&dir).unwrap();
}
