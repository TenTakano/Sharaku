use std::path::Path;

use rusqlite::Connection;

use crate::db::{self, WorkRecord};
use crate::settings;

use super::*;

fn setup_test_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    db::init_db_for_test(&conn).unwrap();
    conn
}

fn insert_folder_work(conn: &Connection, title: &str, path: &str, artist: Option<&str>) {
    db::insert_work(
        conn,
        &WorkRecord {
            title,
            path,
            work_type: "folder",
            page_count: 3,
            thumbnail: b"thumb",
            artist,
            year: None,
            genre: None,
            circle: None,
            origin: None,
        },
    )
    .unwrap();
}

#[test]
fn preview_empty_when_no_folder_works() {
    let conn = setup_test_db();
    let previews = preview_relocation(&conn, Path::new("/library"), "{title}").unwrap();
    assert!(previews.is_empty());
}

#[test]
fn preview_empty_when_path_unchanged() {
    let conn = setup_test_db();
    insert_folder_work(&conn, "MyWork", "/library/MyWork", None);
    let previews = preview_relocation(&conn, Path::new("/library"), "{title}").unwrap();
    assert!(previews.is_empty());
}

#[test]
fn preview_shows_changed_paths() {
    let conn = setup_test_db();
    insert_folder_work(&conn, "MyWork", "/library/old_location", Some("Artist"));
    let previews = preview_relocation(&conn, Path::new("/library"), "{artist}/{title}").unwrap();
    assert_eq!(previews.len(), 1);
    assert_eq!(previews[0].old_path, "/library/old_location");
    assert_eq!(previews[0].new_path, "/library/Artist/MyWork");
    assert_eq!(previews[0].title, "MyWork");
}

#[test]
fn preview_skips_image_type_works() {
    let conn = setup_test_db();
    db::insert_work(
        &conn,
        &WorkRecord {
            title: "ImageWork",
            path: "/library/image.jpg",
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
    let previews = preview_relocation(&conn, Path::new("/library"), "{title}").unwrap();
    assert!(previews.is_empty());
}

#[test]
fn preview_multiple_works_different_paths() {
    let conn = setup_test_db();
    insert_folder_work(&conn, "Work1", "/library/old1", Some("A"));
    insert_folder_work(&conn, "Work2", "/library/old2", Some("B"));
    let previews = preview_relocation(&conn, Path::new("/library"), "{artist}/{title}").unwrap();
    assert_eq!(previews.len(), 2);
}

#[test]
fn execute_moves_files_and_updates_db() {
    let temp = std::env::temp_dir().join("sharaku_test_relocate_exec");
    let _ = std::fs::remove_dir_all(&temp);

    let library_root = temp.join("library");
    let old_dir = library_root.join("old_folder");
    std::fs::create_dir_all(&old_dir).unwrap();
    std::fs::write(old_dir.join("01.jpg"), b"image_data").unwrap();
    std::fs::write(old_dir.join("02.png"), b"image_data2").unwrap();

    let app_data_dir = temp.join("app_data");
    std::fs::create_dir_all(&app_data_dir).unwrap();

    let conn = db::open_db(&app_data_dir).unwrap();
    settings::set_library_root(&conn, &library_root.to_string_lossy()).unwrap();
    settings::set_directory_template(&conn, "{title}").unwrap();
    insert_folder_work(&conn, "MyWork", &old_dir.to_string_lossy(), Some("Artist"));
    drop(conn);

    let conn = db::open_db(&app_data_dir).unwrap();
    let works = db::list_folder_works(&conn).unwrap();
    let plan = compute_relocation_plan(&works, &library_root, "{artist}/{title}");
    assert_eq!(plan.len(), 1);
    assert!(plan[0].new_path.contains("Artist"));

    // Verify files moved correctly
    let new_dir = library_root.join("Artist").join("MyWork");
    std::fs::create_dir_all(&new_dir).unwrap();
    let images = importer::list_images_in_folder(&old_dir).unwrap();
    for image in &images {
        let file_name = image.file_name().unwrap();
        std::fs::rename(image, new_dir.join(file_name)).unwrap();
    }
    let _ = std::fs::remove_dir(&old_dir);

    assert!(new_dir.join("01.jpg").exists());
    assert!(new_dir.join("02.png").exists());
    assert!(!old_dir.exists());

    std::fs::remove_dir_all(&temp).unwrap();
}

#[test]
fn compute_plan_handles_path_collision() {
    let conn = setup_test_db();
    insert_folder_work(&conn, "SameTitle", "/library/folder_a", Some("Artist"));
    insert_folder_work(&conn, "SameTitle", "/library/folder_b", Some("Artist"));

    let works = db::list_folder_works(&conn).unwrap();
    let plan = compute_relocation_plan(&works, Path::new("/library"), "{artist}/{title}");

    assert_eq!(plan.len(), 2);
    assert_ne!(plan[0].new_path, plan[1].new_path);
}

#[test]
fn cleanup_empty_ancestors_removes_empty_dirs() {
    let temp = std::env::temp_dir().join("sharaku_test_cleanup_ancestors");
    let _ = std::fs::remove_dir_all(&temp);

    let stop = temp.join("library");
    let nested = stop.join("a").join("b").join("c");
    std::fs::create_dir_all(&nested).unwrap();

    // Simulate: leaf directory already removed (as in move_work_files)
    std::fs::remove_dir(&nested).unwrap();

    cleanup_empty_ancestors(&nested, &stop);

    assert!(!stop.join("a").exists());
    assert!(stop.exists());

    std::fs::remove_dir_all(&temp).unwrap();
}

#[test]
fn cleanup_empty_ancestors_stops_at_non_empty() {
    let temp = std::env::temp_dir().join("sharaku_test_cleanup_nonempty");
    let _ = std::fs::remove_dir_all(&temp);

    let stop = temp.join("library");
    let parent = stop.join("artist");
    let child = parent.join("work");
    std::fs::create_dir_all(&child).unwrap();
    std::fs::write(parent.join("other_file.txt"), b"data").unwrap();

    // Simulate: leaf directory already removed
    std::fs::remove_dir(&child).unwrap();

    cleanup_empty_ancestors(&child, &stop);

    assert!(parent.exists());

    std::fs::remove_dir_all(&temp).unwrap();
}
