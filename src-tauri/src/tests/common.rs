use rusqlite::Connection;

use crate::db;
use crate::library;

/// インメモリDBを開き、テスト用ライブラリを1件追加してIDを test_library_id に固定する。
/// テストヘルパー共通化: tests/{db, relocator, integrity, settings}.rs で重複していた
/// セットアップをここに集約する。
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
