use rusqlite::Connection;

use crate::{Error, Result};

const SCHEMA_VERSION: i32 = 8;

pub fn run_migrations(conn: &mut Connection) -> Result<()> {
    ensure_migration_table(conn)?;

    let current_version = get_schema_version(conn)?;

    if current_version > SCHEMA_VERSION {
        return Err(Error::Internal(format!(
            "Database schema version ({}) is newer than supported version ({}). Please update the \
             plugin.",
            current_version, SCHEMA_VERSION
        )));
    }

    for version in (current_version + 1)..=SCHEMA_VERSION {
        apply_migration(conn, version).map_err(|e| {
            Error::Internal(format!("Failed to apply migration {}: {}", version, e))
        })?;
    }

    Ok(())
}

fn ensure_migration_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS migration (
            id INTEGER PRIMARY KEY
        )",
        [],
    )?;
    Ok(())
}

fn get_schema_version(conn: &Connection) -> Result<i32> {
    let version = conn.query_row("SELECT COALESCE(MAX(id), 0) FROM migration", [], |row| {
        row.get(0)
    })?;
    Ok(version)
}

fn set_schema_version(conn: &Connection, version: i32) -> Result<()> {
    conn.execute("INSERT INTO migration (id) VALUES (?1)", [version])?;
    Ok(())
}

fn apply_migration(conn: &mut Connection, version: i32) -> Result<()> {
    let tx = conn.transaction()?;

    match version {
        1 => migration_v1(&tx)?,
        2 => migration_v2(&tx)?,
        3 => migration_v3(&tx)?,
        4 => migration_v4(&tx)?,
        5 => migration_v5(&tx)?,
        6 => migration_v6(&tx)?,
        7 => migration_v7(&tx)?,
        8 => migration_v8(&tx)?,
        _ => {
            return Err(Error::Internal(format!(
                "Unknown migration version: {}",
                version
            )));
        }
    }

    set_schema_version(&tx, version)?;
    tx.commit()?;

    Ok(())
}

fn migration_v1(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE play_time(
            date_time TEXT,
            duration INT,
            game_id TEXT
        );

        CREATE TABLE overall_time(
            game_id TEXT PRIMARY KEY,
            duration INT
        );

        CREATE TABLE game_dict(
            game_id TEXT PRIMARY KEY,
            name TEXT
        );
        "#,
    )?;
    Ok(())
}

fn migration_v2(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE INDEX play_time_date_time_epoch_idx
            ON play_time(STRFTIME('%s', date_time));

        CREATE INDEX play_time_game_id_idx
            ON play_time(game_id);

        CREATE INDEX overall_time_game_id_idx
            ON overall_time(game_id);
        "#,
    )?;
    Ok(())
}

fn migration_v3(conn: &Connection) -> Result<()> {
    conn.execute("ALTER TABLE play_time ADD COLUMN migrated TEXT", [])?;
    Ok(())
}

fn migration_v4(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        DROP INDEX play_time_date_time_epoch_idx;

        CREATE INDEX play_time_date_time_epoch_idx
            ON play_time(STRFTIME('%s', date_time));
        "#,
    )?;
    Ok(())
}

fn migration_v5(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE game_file_checksum(
            checksum_id INTEGER PRIMARY KEY AUTOINCREMENT,
            game_id TEXT NOT NULL,
            checksum TEXT NOT NULL,
            algorithm TEXT NOT NULL CHECK(algorithm IN (
                'BLAKE2B', 'BLAKE2S',
                'SHA224', 'SHA256', 'SHA384', 'SHA512', 'SHA512_224', 'SHA512_256',
                'SHA3_224', 'SHA3_256', 'SHA3_384', 'SHA3_512',
                'SHAKE_128', 'SHAKE_256'
            )),
            chunk_size INTEGER NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (game_id) REFERENCES game_dict(game_id),
            UNIQUE (game_id, checksum, algorithm)
        );

        CREATE INDEX game_file_checksum_checksum_algorithm_idx
            ON game_file_checksum(checksum, algorithm);
        "#,
    )?;
    Ok(())
}

fn migration_v6(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        DROP INDEX IF EXISTS overall_time_game_id_idx;
        DROP INDEX IF EXISTS play_time_game_id_idx;
        DROP INDEX IF EXISTS play_time_date_time_epoch_idx;

        CREATE INDEX IF NOT EXISTS play_time_date_time_idx
            ON play_time(date_time);

        CREATE INDEX IF NOT EXISTS play_time_game_id_date_time_idx
            ON play_time(game_id, date_time);
        "#,
    )?;
    Ok(())
}

fn migration_v7(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE INDEX IF NOT EXISTS idx_overall_time_game_id
            ON overall_time(game_id);

        CREATE INDEX IF NOT EXISTS idx_game_dict_game_id
            ON game_dict(game_id);

        CREATE INDEX IF NOT EXISTS idx_play_time_migrated
            ON play_time(migrated) WHERE migrated IS NULL;

        CREATE INDEX IF NOT EXISTS idx_game_file_checksum_game_id
            ON game_file_checksum(game_id);

        CREATE INDEX IF NOT EXISTS idx_game_file_checksum_composite
            ON game_file_checksum(game_id, checksum, algorithm);
        "#,
    )?;
    Ok(())
}

fn migration_v8(conn: &Connection) -> Result<()> {
    conn.execute(
        "DELETE FROM game_file_checksum
         WHERE game_id NOT IN (SELECT game_id FROM game_dict)",
        [],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;

    const EXPECTED_TABLES: &[&str] = &[
        "play_time",
        "overall_time",
        "game_dict",
        "game_file_checksum",
        "migration",
    ];

    #[test]
    fn test_full_migration_sequence() {
        let mut conn = Connection::open_in_memory().unwrap();

        run_migrations(&mut conn).unwrap();

        let version = get_schema_version(&conn).unwrap();
        assert_eq!(
            version, SCHEMA_VERSION,
            "Schema version should match expected"
        );

        for &table_name in EXPECTED_TABLES {
            assert!(
                table_exists(&conn, table_name),
                "Table '{}' should exist after migrations",
                table_name
            );
        }

        assert!(
            column_exists(&conn, "play_time", "migrated"),
            "play_time should have migrated column"
        );
    }

    #[test]
    fn test_incremental_migrations() {
        let mut conn = Connection::open_in_memory().unwrap();
        ensure_migration_table(&conn).unwrap();

        for expected_version in 1..=SCHEMA_VERSION {
            apply_migration(&mut conn, expected_version).unwrap();

            let actual_version = get_schema_version(&conn).unwrap();

            assert_eq!(
                actual_version, expected_version,
                "Schema version should be {} after migration {}",
                expected_version, expected_version
            );
        }
    }

    #[test]
    fn test_migration_idempotency() {
        let mut conn = Connection::open_in_memory().unwrap();

        // Run migrations twice
        run_migrations(&mut conn).unwrap();
        run_migrations(&mut conn).unwrap();

        let version = get_schema_version(&conn).unwrap();
        assert_eq!(version, SCHEMA_VERSION, "Version should remain stable");
    }

    #[test]
    fn test_future_schema_version_error() {
        let conn = Connection::open_in_memory().unwrap();

        conn.execute("CREATE TABLE migration (id INTEGER PRIMARY KEY)", [])
            .unwrap();
        conn.execute(
            "INSERT INTO migration (id) VALUES (?1)",
            [SCHEMA_VERSION + 100],
        )
        .unwrap();

        let mut conn = conn; // Make mutable for migration call
        let result = run_migrations(&mut conn);

        assert!(result.is_err(), "Should error on future schema version");

        let error_msg = result.unwrap_err().to_string();

        assert!(
            error_msg.contains("newer than supported"),
            "Error should mention version incompatibility, got: {}",
            error_msg
        );
    }

    #[test]
    fn test_migration_atomicity() {
        let mut conn = Connection::open_in_memory().unwrap();
        ensure_migration_table(&conn).unwrap();

        for v in 1..=3 {
            apply_migration(&mut conn, v).unwrap();
        }

        let version_before = get_schema_version(&conn).unwrap();
        assert_eq!(version_before, 3);

        assert!(table_exists(&conn, "play_time"));
        assert!(column_exists(&conn, "play_time", "migrated"));
    }

    fn table_exists(conn: &Connection, table_name: &str) -> bool {
        conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master
             WHERE type = 'table' AND name = ?1",
            [table_name],
            |row| row.get(0),
        )
        .unwrap_or(false)
    }

    fn column_exists(conn: &Connection, table_name: &str, column_name: &str) -> bool {
        conn.query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info(?1)
             WHERE name = ?2",
            [table_name, column_name],
            |row| row.get(0),
        )
        .unwrap_or(false)
    }
}
