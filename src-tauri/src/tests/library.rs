use super::*;

fn temp_store(name: &str) -> LibraryStore {
    let dir = std::env::temp_dir().join(format!("sharaku_test_library_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    LibraryStore::new(&dir)
}

#[test]
fn load_empty_returns_empty() {
    let store = temp_store("load_empty");
    let (libs, active) = store.load().unwrap();
    assert!(libs.is_empty());
    assert!(active.is_none());
}

#[test]
fn add_library_creates_entry() {
    let store = temp_store("add");
    let lib = store.add("Photos", "/path/to/photos").unwrap();
    assert_eq!(lib.name, "Photos");
    assert_eq!(lib.path, "/path/to/photos");
    assert!(!lib.id.is_empty());

    let (libs, active) = store.load().unwrap();
    assert_eq!(libs.len(), 1);
    assert_eq!(active.as_deref(), Some(lib.id.as_str()));
}

#[test]
fn add_duplicate_path_returns_error() {
    let store = temp_store("dup_path");
    store.add("Photos", "/same/path").unwrap();
    let result = store.add("Another", "/same/path");
    assert!(result.is_err());
}

#[test]
fn add_second_library_keeps_first_active() {
    let store = temp_store("second");
    let first = store.add("First", "/first").unwrap();
    store.add("Second", "/second").unwrap();

    let (_, active) = store.load().unwrap();
    assert_eq!(active.as_deref(), Some(first.id.as_str()));
}

#[test]
fn remove_library() {
    let store = temp_store("remove");
    let lib = store.add("ToRemove", "/remove").unwrap();
    store.remove(&lib.id).unwrap();

    let (libs, _) = store.load().unwrap();
    assert!(libs.is_empty());
}

#[test]
fn remove_active_library_activates_next() {
    let store = temp_store("remove_active");
    let first = store.add("First", "/first").unwrap();
    let second = store.add("Second", "/second").unwrap();

    store.remove(&first.id).unwrap();
    let (libs, active) = store.load().unwrap();
    assert_eq!(libs.len(), 1);
    assert_eq!(active.as_deref(), Some(second.id.as_str()));
}

#[test]
fn remove_nonexistent_returns_error() {
    let store = temp_store("remove_nonexistent");
    let result = store.remove("fake_id");
    assert!(result.is_err());
}

#[test]
fn set_active() {
    let store = temp_store("set_active");
    store.add("First", "/first").unwrap();
    let second = store.add("Second", "/second").unwrap();

    store.set_active(&second.id).unwrap();
    let (_, active) = store.load().unwrap();
    assert_eq!(active.as_deref(), Some(second.id.as_str()));
}

#[test]
fn set_active_nonexistent_returns_error() {
    let store = temp_store("set_active_nonexistent");
    let result = store.set_active("fake_id");
    assert!(result.is_err());
}

#[test]
fn find_by_id_found() {
    let store = temp_store("find");
    let lib = store.add("Test", "/test").unwrap();
    let found = store.find_by_id(&lib.id).unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Test");
}

#[test]
fn find_by_id_not_found() {
    let store = temp_store("find_not_found");
    let found = store.find_by_id("fake_id").unwrap();
    assert!(found.is_none());
}

#[test]
fn active_library_returns_active() {
    let store = temp_store("active");
    let lib = store.add("Active", "/active").unwrap();
    let active = store.active_library().unwrap();
    assert!(active.is_some());
    assert_eq!(active.unwrap().id, lib.id);
}

#[test]
fn active_library_returns_none_when_empty() {
    let store = temp_store("active_empty");
    let active = store.active_library().unwrap();
    assert!(active.is_none());
}
