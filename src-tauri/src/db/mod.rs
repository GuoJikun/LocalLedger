//! 数据库连接、版本化迁移与设备身份初始化。

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{now, Device};
use crate::error::{AppError, AppResult};

const MIGRATIONS: &[(&str, &str)] = &[("1", include_str!("migrations/001_init.sql"))];
const SCHEMA_VERSION_KEY: &str = "schema_version";
const DEVICE_ID_KEY: &str = "device_id";

pub fn open(path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;
         PRAGMA busy_timeout = 5000;",
    )?;
    Ok(conn)
}

#[cfg(test)]
pub fn open_in_memory() -> AppResult<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}

/// 按 `meta.schema_version` 顺序执行尚未应用的迁移。
pub fn migrate(conn: &Connection) -> AppResult<()> {
    // 引导表：迁移版本号本身存在 meta 中，建表必须先于版本查询。
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS meta (
             key   TEXT PRIMARY KEY,
             value TEXT NOT NULL
         );",
    )?;

    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(CAST(value AS INTEGER)), 0) FROM meta WHERE key = ?1",
            [SCHEMA_VERSION_KEY],
            |row| row.get(0),
        )
        .optional()?
        .unwrap_or(0);

    for (version, sql) in MIGRATIONS {
        let target: i64 = version
            .parse()
            .map_err(|_| AppError::Invalid(format!("非法迁移版本: {version}")))?;
        if target <= current {
            continue;
        }
        let tx = conn.unchecked_transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![SCHEMA_VERSION_KEY, version],
        )?;
        tx.commit()?;
    }

    Ok(())
}

/// 设备 ID 持久化到 `meta`，设备信息写入 `devices`，重启后保持不变。
pub fn ensure_device(conn: &Connection) -> AppResult<Device> {
    let name = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "本机".to_string());
    let timestamp = now();

    let stored_id: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = ?1",
            [DEVICE_ID_KEY],
            |row| row.get(0),
        )
        .optional()?;

    let id = match stored_id {
        Some(id) => id,
        None => {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO meta (key, value) VALUES (?1, ?2)",
                params![DEVICE_ID_KEY, id],
            )?;
            id
        }
    };

    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM devices WHERE id = ?1)",
            [&id],
            |row| row.get::<_, i64>(0),
        )
        .map(|value| value != 0)?;

    if exists {
        conn.execute(
            "UPDATE devices SET name = ?2, last_seen_at = ?3 WHERE id = ?1",
            params![id, name, timestamp],
        )?;
    } else {
        conn.execute(
            "INSERT INTO devices (id, name, last_seen_at) VALUES (?1, ?2, ?3)",
            params![id, name, timestamp],
        )?;
    }

    Ok(Device {
        id,
        name,
        last_seen_at: timestamp,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_creates_schema_and_seeds_defaults() {
        let conn = open_in_memory().unwrap();
        migrate(&conn).unwrap();
        migrate(&conn).unwrap(); // 幂等

        let version: String = conn
            .query_row(
                "SELECT value FROM meta WHERE key = 'schema_version'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(version, "1");

        let accounts: i64 = conn
            .query_row("SELECT COUNT(*) FROM accounts", [], |r| r.get(0))
            .unwrap();
        let categories: i64 = conn
            .query_row("SELECT COUNT(*) FROM categories", [], |r| r.get(0))
            .unwrap();
        assert_eq!(accounts, 1);
        assert_eq!(categories, 5);
    }

    #[test]
    fn device_id_is_stable_across_calls() {
        let conn = open_in_memory().unwrap();
        migrate(&conn).unwrap();

        let first = ensure_device(&conn).unwrap();
        let second = ensure_device(&conn).unwrap();
        assert_eq!(first.id, second.id);
        assert!(!first.id.is_empty());

        let devices: i64 = conn
            .query_row("SELECT COUNT(*) FROM devices", [], |r| r.get(0))
            .unwrap();
        assert_eq!(devices, 1);
    }
}
