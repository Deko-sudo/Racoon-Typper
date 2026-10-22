use racoon_data::{backup, Database};
use rusqlite::Connection;
use std::fs;

#[test]
fn incompatible_or_modified_backups_never_replace_history() {
    let mut failures = Vec::new();
    for case in ["foreign-schema", "missing-trigger", "changed-literal"] {
        let root = std::env::temp_dir().join(format!(
            "racoon-restore-contract-{}-{case}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        let live = root.join("live.db");
        let source = root.join("backup.db");
        let db = Database::open(&live).unwrap();
        db.with_connection(|conn| {conn.execute("INSERT INTO custom_texts(name,text,created_at) VALUES('original-history','body','2026-01-01T00:00:00Z')",[]).unwrap();Ok(())}).unwrap();
        if case == "foreign-schema" {
            Connection::open(&source)
                .unwrap()
                .execute_batch("CREATE TABLE tests(id INTEGER PRIMARY KEY)")
                .unwrap();
        } else {
            backup::backup_to_path(&db, &source).unwrap();
            let conn = Connection::open(&source).unwrap();
            let sql:String=conn.query_row("SELECT sql FROM sqlite_schema WHERE name='session_ledger_terminal_state_immutable'",[],|row|row.get(0)).unwrap();
            conn.execute_batch("DROP TRIGGER session_ledger_terminal_state_immutable")
                .unwrap();
            if case == "changed-literal" {
                conn.execute_batch(&sql.replace("'finalized'", "'final ized'"))
                    .unwrap();
            }
        }
        drop(db);
        let result = backup::restore_from_path(&source, &live);
        if result.is_ok() {
            failures.push(format!("{case}: incompatible backup accepted"));
        }
        let conn = Connection::open(&live).unwrap();
        let history = conn.query_row(
            "SELECT COUNT(*) FROM custom_texts WHERE name='original-history'",
            [],
            |row| row.get::<_, i64>(0),
        );
        if !matches!(history, Ok(1)) {
            failures.push(format!("{case}: history lost: {history:?}"));
        }
        drop(conn);
        fs::remove_dir_all(&root).unwrap();
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
