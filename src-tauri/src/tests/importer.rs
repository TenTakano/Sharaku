use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tauri::ipc::Channel;
use tempfile::TempDir;

use super::*;
use crate::test_common::test_db_with_library;

fn write_test_image(path: &Path) {
    let img = image::RgbImage::new(2, 2);
    img.save(path).unwrap();
}

fn make_request(source_path: &Path, title: &str, mode: ImportMode) -> ImportRequest {
    ImportRequest {
        source_path: source_path.to_string_lossy().to_string(),
        title: title.to_string(),
        artist: None,
        year: None,
        genre: None,
        circle: None,
        origin: None,
        mode,
        kind: ImportKind::Image,
    }
}

fn noop_progress_channel() -> Channel<DiscoverProgress> {
    Channel::new(|_| Ok(()))
}

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
    let dir = TempDir::new().unwrap();
    let dir = dir.path();

    std::fs::write(dir.join("01.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("02.png"), b"fake").unwrap();
    std::fs::write(dir.join("readme.txt"), b"text").unwrap();

    let images = list_images_in_folder(dir).unwrap();
    assert_eq!(images.len(), 2);
    assert!(
        images[0].file_name().unwrap().to_str().unwrap()
            <= images[1].file_name().unwrap().to_str().unwrap()
    );
}

#[test]
fn list_images_empty_folder() {
    let dir = TempDir::new().unwrap();

    let images = list_images_in_folder(dir.path()).unwrap();
    assert!(images.is_empty());
}

#[test]
fn list_images_sorted_order() {
    let dir = TempDir::new().unwrap();
    let dir = dir.path();

    std::fs::write(dir.join("c.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("a.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("b.jpg"), b"fake").unwrap();

    let images = list_images_in_folder(dir).unwrap();
    assert_eq!(images.len(), 3);
    assert_eq!(images[0].file_name().unwrap(), "a.jpg");
    assert_eq!(images[1].file_name().unwrap(), "b.jpg");
    assert_eq!(images[2].file_name().unwrap(), "c.jpg");
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
    let dir = TempDir::new().unwrap();
    let dir = dir.path();

    std::fs::write(dir.join("page1.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("page2.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("page10.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("page20.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("page3.jpg"), b"fake").unwrap();

    let images = list_images_in_folder(dir).unwrap();
    assert_eq!(images.len(), 5);
    assert_eq!(images[0].file_name().unwrap(), "page1.jpg");
    assert_eq!(images[1].file_name().unwrap(), "page2.jpg");
    assert_eq!(images[2].file_name().unwrap(), "page3.jpg");
    assert_eq!(images[3].file_name().unwrap(), "page10.jpg");
    assert_eq!(images[4].file_name().unwrap(), "page20.jpg");
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
    let dir = TempDir::new().unwrap();
    let dir = dir.path();

    std::fs::write(dir.join("a.jpg"), b"fake").unwrap();
    std::fs::write(dir.join("b.png"), b"fake").unwrap();
    std::fs::write(dir.join("readme.txt"), b"text").unwrap();

    let sub = dir.join("subdir");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(sub.join("c.jpg"), b"fake").unwrap();

    assert_eq!(crate::scanner::count_direct_images(dir), 2);
}

#[test]
fn count_direct_images_empty_dir() {
    let dir = TempDir::new().unwrap();

    assert_eq!(crate::scanner::count_direct_images(dir.path()), 0);
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
fn import_single_image_metadata_only_registers_source_path_without_copy() {
    const LIB_ID: &str = "test_lib_import_single_metadata_only";
    let temp = TempDir::new().unwrap();
    let library_root = temp.path().join("library");
    std::fs::create_dir_all(&library_root).unwrap();
    let source = temp.path().join("source.png");
    write_test_image(&source);

    let conn = test_db_with_library(LIB_ID);
    crate::settings::set_resource_mode(&conn, LIB_ID, "metadata_only").unwrap();

    let request = make_request(&source, "MyImage", ImportMode::Copy);
    let result = import_single_image(&request, &conn, LIB_ID, &library_root).unwrap();

    assert_eq!(result.destination_path, source.to_string_lossy());
    assert_eq!(result.page_count, 1);
    assert!(source.exists());

    let works = crate::db::list_works(&conn, LIB_ID, "created_at", "desc").unwrap();
    assert_eq!(works.len(), 1);
}

#[test]
fn import_single_image_full_copy_creates_file_and_keeps_source() {
    const LIB_ID: &str = "test_lib_import_single_full_copy";
    let temp = TempDir::new().unwrap();
    let library_root = temp.path().join("library");
    std::fs::create_dir_all(&library_root).unwrap();
    let source = temp.path().join("source.png");
    write_test_image(&source);

    let conn = test_db_with_library(LIB_ID);
    crate::settings::set_directory_template(&conn, LIB_ID, "{title}").unwrap();

    let request = make_request(&source, "MyImage", ImportMode::Copy);
    let result = import_single_image(&request, &conn, LIB_ID, &library_root).unwrap();

    let dest = Path::new(&result.destination_path);
    assert!(dest.exists());
    assert!(dest.starts_with(library_root.join("pictures")));
    assert!(source.exists());

    let works = crate::db::list_works(&conn, LIB_ID, "created_at", "desc").unwrap();
    assert_eq!(works.len(), 1);
}

#[test]
fn import_single_image_move_removes_source_file() {
    const LIB_ID: &str = "test_lib_import_single_move";
    let temp = TempDir::new().unwrap();
    let library_root = temp.path().join("library");
    std::fs::create_dir_all(&library_root).unwrap();
    let source = temp.path().join("source.png");
    write_test_image(&source);

    let conn = test_db_with_library(LIB_ID);
    crate::settings::set_directory_template(&conn, LIB_ID, "{title}").unwrap();

    let request = make_request(&source, "MyImage", ImportMode::Move);
    let result = import_single_image(&request, &conn, LIB_ID, &library_root).unwrap();

    let dest = Path::new(&result.destination_path);
    assert!(dest.exists());
    assert!(!source.exists());
}

#[test]
fn import_single_image_rolls_back_dest_dir_on_db_insert_failure() {
    const LIB_ID: &str = "test_lib_import_single_rollback";
    let temp = TempDir::new().unwrap();
    let library_root = temp.path().join("library");
    std::fs::create_dir_all(&library_root).unwrap();
    let source = temp.path().join("source.png");
    write_test_image(&source);

    let conn = test_db_with_library(LIB_ID);
    crate::settings::set_directory_template(&conn, LIB_ID, "{title}").unwrap();

    let request = make_request(&source, "MyImage", ImportMode::Copy);

    // Pre-register a work whose path collides with the path import_single_image will
    // compute (resolve_unique_work_path only checks filesystem existence, not the DB,
    // so it cannot detect this in advance). This forces the UNIQUE(library_id, path)
    // constraint to fail on insert, exercising the rollback branch.
    let dest_dir = template::resolve_unique_work_path(
        &library_root,
        "{title}",
        &WorkMetadata {
            title: "MyImage".to_string(),
            artist: None,
            year: None,
            genre: None,
            circle: None,
            origin: None,
            work_type: None,
        },
        template::WORK_KIND_IMAGE,
    );
    let colliding_path = dest_dir.join("source.png");
    crate::db::insert_work(
        &conn,
        &crate::db::WorkRecord {
            library_id: LIB_ID,
            title: "Existing",
            path: &colliding_path.to_string_lossy(),
            work_type: "image",
            page_count: 1,
            thumbnail: b"thumb",
            artist: None,
            year: None,
            genre: None,
            circle: None,
            origin: None,
        },
    )
    .unwrap();

    let result = import_single_image(&request, &conn, LIB_ID, &library_root);
    assert!(result.is_err());
    assert!(!dest_dir.exists());
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

#[test]
fn discover_from_paths_handles_mixed_folder_and_image_roots() {
    const LIB_ID: &str = "test_lib_discover_mixed";
    let temp = TempDir::new().unwrap();

    let folder_root = temp.path().join("folder1");
    std::fs::create_dir_all(&folder_root).unwrap();
    std::fs::write(folder_root.join("01.jpg"), b"a").unwrap();
    std::fs::write(folder_root.join("02.jpg"), b"b").unwrap();

    let image_root = temp.path().join("standalone.png");
    std::fs::write(&image_root, b"c").unwrap();

    let conn = test_db_with_library(LIB_ID);
    let roots = vec![folder_root.clone(), image_root.clone()];
    let result = discover_from_paths(&roots, &conn, LIB_ID, &noop_progress_channel()).unwrap();

    assert_eq!(result.folders.len(), 1);
    assert_eq!(result.folders[0].path, folder_root.to_string_lossy());
    assert_eq!(result.images.len(), 1);
    assert_eq!(result.images[0].path, image_root.to_string_lossy());
    assert!(result.skipped_folders.is_empty());
}

#[test]
fn discover_from_paths_marks_already_registered_entries() {
    const LIB_ID: &str = "test_lib_discover_already_registered";
    let temp = TempDir::new().unwrap();

    let registered_folder = temp.path().join("folder_registered");
    std::fs::create_dir_all(&registered_folder).unwrap();
    std::fs::write(registered_folder.join("01.jpg"), b"a").unwrap();

    let unregistered_folder = temp.path().join("folder_unregistered");
    std::fs::create_dir_all(&unregistered_folder).unwrap();
    std::fs::write(unregistered_folder.join("01.jpg"), b"a").unwrap();

    let registered_image = temp.path().join("registered.png");
    std::fs::write(&registered_image, b"a").unwrap();

    let unregistered_image = temp.path().join("unregistered.png");
    std::fs::write(&unregistered_image, b"a").unwrap();

    let conn = test_db_with_library(LIB_ID);
    db::insert_work(
        &conn,
        &WorkRecord {
            library_id: LIB_ID,
            title: "Existing Folder",
            path: &registered_folder.to_string_lossy(),
            work_type: "folder",
            page_count: 1,
            thumbnail: b"thumb",
            artist: None,
            year: None,
            genre: None,
            circle: None,
            origin: None,
        },
    )
    .unwrap();
    db::insert_work(
        &conn,
        &WorkRecord {
            library_id: LIB_ID,
            title: "Existing Image",
            path: &registered_image.to_string_lossy(),
            work_type: "image",
            page_count: 1,
            thumbnail: b"thumb",
            artist: None,
            year: None,
            genre: None,
            circle: None,
            origin: None,
        },
    )
    .unwrap();

    let roots = vec![
        registered_folder.clone(),
        unregistered_folder.clone(),
        registered_image.clone(),
        unregistered_image.clone(),
    ];
    let result = discover_from_paths(&roots, &conn, LIB_ID, &noop_progress_channel()).unwrap();

    let folder = result
        .folders
        .iter()
        .find(|f| f.path == registered_folder.to_string_lossy())
        .unwrap();
    assert!(folder.already_registered);

    let folder = result
        .folders
        .iter()
        .find(|f| f.path == unregistered_folder.to_string_lossy())
        .unwrap();
    assert!(!folder.already_registered);

    let image = result
        .images
        .iter()
        .find(|i| i.path == registered_image.to_string_lossy())
        .unwrap();
    assert!(image.already_registered);

    let image = result
        .images
        .iter()
        .find(|i| i.path == unregistered_image.to_string_lossy())
        .unwrap();
    assert!(!image.already_registered);
}

#[test]
fn discover_from_paths_dedupes_duplicate_roots() {
    struct Case {
        lib_id: &'static str,
        setup: fn(&Path) -> PathBuf,
        assert_result: fn(&DiscoverResult),
    }

    let cases = [
        Case {
            lib_id: "test_lib_discover_dedup_folder",
            setup: |dir| {
                let folder_root = dir.join("folder1");
                std::fs::create_dir_all(&folder_root).unwrap();
                std::fs::write(folder_root.join("01.jpg"), b"a").unwrap();
                folder_root
            },
            assert_result: |result| assert_eq!(result.folders.len(), 1),
        },
        Case {
            lib_id: "test_lib_discover_dedup_image",
            setup: |dir| {
                let image_root = dir.join("standalone.png");
                std::fs::write(&image_root, b"a").unwrap();
                image_root
            },
            assert_result: |result| assert_eq!(result.images.len(), 1),
        },
    ];

    for case in cases {
        let temp = TempDir::new().unwrap();
        let root = (case.setup)(temp.path());

        let conn = test_db_with_library(case.lib_id);
        let roots = vec![root.clone(), root.clone()];
        let result =
            discover_from_paths(&roots, &conn, case.lib_id, &noop_progress_channel()).unwrap();

        (case.assert_result)(&result);
    }
}

#[test]
fn discover_from_paths_skips_unsupported_and_missing_paths() {
    const LIB_ID: &str = "test_lib_discover_skips";
    let temp = TempDir::new().unwrap();

    let text_file = temp.path().join("note.txt");
    std::fs::write(&text_file, b"not an image").unwrap();
    let missing_path = temp.path().join("does_not_exist");

    let conn = test_db_with_library(LIB_ID);
    let roots = vec![text_file, missing_path];
    let result = discover_from_paths(&roots, &conn, LIB_ID, &noop_progress_channel()).unwrap();

    assert!(result.folders.is_empty());
    assert!(result.images.is_empty());
    assert!(result.skipped_folders.is_empty());
}

#[test]
fn discover_from_paths_sends_completed_progress_with_total_found() {
    const LIB_ID: &str = "test_lib_discover_progress";
    let temp = TempDir::new().unwrap();

    let folder_root = temp.path().join("folder1");
    std::fs::create_dir_all(&folder_root).unwrap();
    std::fs::write(folder_root.join("01.jpg"), b"a").unwrap();

    let image_root = temp.path().join("standalone.png");
    std::fs::write(&image_root, b"a").unwrap();

    let conn = test_db_with_library(LIB_ID);
    let messages: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
    let messages_clone = messages.clone();
    let channel = Channel::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            let progress: serde_json::Value = serde_json::from_str(&json).unwrap();
            messages_clone.lock().unwrap().push(progress);
        }
        Ok(())
    });

    let roots = vec![folder_root, image_root];
    discover_from_paths(&roots, &conn, LIB_ID, &channel).unwrap();

    let received = messages.lock().unwrap();
    let last = received
        .last()
        .expect("expected at least one progress message");
    assert_eq!(last["type"], "completed");
    assert_eq!(last["found"], 2);
}
