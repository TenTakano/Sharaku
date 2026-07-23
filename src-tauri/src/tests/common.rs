use rusqlite::Connection;

use crate::db;
use crate::library;

/// Opens an in-memory DB, adds a single test library, and pins its ID to test_library_id.
/// Test helper consolidation: gathers here the setup that was duplicated across
/// tests/{db, relocator, integrity, settings}.rs.
pub(crate) fn test_db_with_library(test_library_id: &str) -> Connection {
    let conn = db::open_db_in_memory().unwrap();
    library::add_library(&conn, "Test", Some("/test")).ok();
    conn.execute(
        "UPDATE libraries SET id = ?1 WHERE id = (SELECT id FROM libraries LIMIT 1)",
        [test_library_id],
    )
    .unwrap();
    conn.execute(
        "UPDATE libraries SET is_active = 1 WHERE id = ?1",
        [test_library_id],
    )
    .unwrap();
    conn
}
