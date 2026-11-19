//! Repository 的 SQLite 实现：所有写操作都在事务内追加 `sync_changes`。

use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row, ToSql};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::{
    now, Account, Category, CategoryPatch, EntityType, NewCategory, NewTransaction, SyncChange,
    SyncOp, Transaction, TransactionKind, TransactionPatch, TransactionSource,
};
use crate::error::{AppError, AppResult};

use super::{AccountRepository, CategoryRepository, SyncChangeRepository, TransactionRepository};

const TRANSACTION_COLUMNS: &str =
    "id, account_id, source, external_id, kind, amount, counterparty, category_id, note, \
     happened_at, payment_method, status, raw_data, created_at, updated_at, device_id, version, deleted";

const CATEGORY_COLUMNS: &str =
    "id, name, parent_id, icon, color, sort_order, created_at, updated_at, device_id, version, deleted";

fn sql_conversion(error: AppError) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

fn bit(value: i64) -> bool {
    value != 0
}

fn map_transaction(row: &Row<'_>) -> rusqlite::Result<Transaction> {
    let source =
        TransactionSource::parse(&row.get::<_, String>("source")?).map_err(sql_conversion)?;
    let kind = TransactionKind::parse(&row.get::<_, String>("kind")?).map_err(sql_conversion)?;
    Ok(Transaction {
        id: row.get("id")?,
        account_id: row.get("account_id")?,
        source,
        external_id: row.get("external_id")?,
        kind,
        amount: row.get("amount")?,
        counterparty: row.get("counterparty")?,
        category_id: row.get("category_id")?,
        note: row.get("note")?,
        happened_at: row.get("happened_at")?,
        payment_method: row.get("payment_method")?,
        status: row.get("status")?,
        raw_data: row.get("raw_data")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        device_id: row.get("device_id")?,
        version: row.get("version")?,
        deleted: bit(row.get("deleted")?),
    })
}

fn map_category(row: &Row<'_>) -> rusqlite::Result<Category> {
    Ok(Category {
        id: row.get("id")?,
        name: row.get("name")?,
        parent_id: row.get("parent_id")?,
        icon: row.get("icon")?,
        color: row.get("color")?,
        sort_order: row.get("sort_order")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        device_id: row.get("device_id")?,
        version: row.get("version")?,
        deleted: bit(row.get("deleted")?),
    })
}

fn map_sync_change(row: &Row<'_>) -> rusqlite::Result<SyncChange> {
    let entity_type =
        EntityType::parse(&row.get::<_, String>("entity_type")?).map_err(sql_conversion)?;
    let op = SyncOp::parse(&row.get::<_, String>("op")?).map_err(sql_conversion)?;
    Ok(SyncChange {
        seq: row.get("seq")?,
        entity_type,
        entity_id: row.get("entity_id")?,
        op,
        payload: row.get("payload")?,
        created_at: row.get("created_at")?,
        device_id: row.get("device_id")?,
    })
}

fn get_transaction(
    conn: &Connection,
    id: &str,
    include_deleted: bool,
) -> AppResult<Option<Transaction>> {
    let sql = format!(
        "SELECT {TRANSACTION_COLUMNS} FROM transactions WHERE id = ?1{}",
        if include_deleted {
            ""
        } else {
            " AND deleted = 0"
        }
    );
    Ok(conn.query_row(&sql, [id], map_transaction).optional()?)
}

fn get_category(conn: &Connection, id: &str, include_deleted: bool) -> AppResult<Option<Category>> {
    let sql = format!(
        "SELECT {CATEGORY_COLUMNS} FROM categories WHERE id = ?1{}",
        if include_deleted {
            ""
        } else {
            " AND deleted = 0"
        }
    );
    Ok(conn.query_row(&sql, [id], map_category).optional()?)
}

fn record_change(
    conn: &Connection,
    entity_type: EntityType,
    entity_id: &str,
    op: SyncOp,
    payload: Option<&str>,
    timestamp: &str,
    device_id: &str,
) -> AppResult<()> {
    conn.execute(
        "INSERT INTO sync_changes (entity_type, entity_id, op, payload, created_at, device_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            entity_type.as_str(),
            entity_id,
            op.as_str(),
            payload,
            timestamp,
            device_id
        ],
    )?;
    Ok(())
}

fn serialize_payload<T: Serialize>(value: &T) -> AppResult<String> {
    serde_json::to_string(value).map_err(|error| AppError::Invalid(error.to_string()))
}

pub struct SqliteRepository<'a> {
    conn: &'a Connection,
    device_id: &'a str,
}

impl<'a> SqliteRepository<'a> {
    pub fn new(conn: &'a Connection, device_id: &'a str) -> Self {
        Self { conn, device_id }
    }
}

impl TransactionRepository for SqliteRepository<'_> {
    fn insert_transaction(&self, input: &NewTransaction) -> AppResult<Transaction> {
        if input.external_id.trim().is_empty() {
            return Err(AppError::Invalid("external_id 不能为空".to_string()));
        }
        if input.account_id.trim().is_empty() {
            return Err(AppError::Invalid("account_id 不能为空".to_string()));
        }

        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO transactions
             (id, account_id, source, external_id, kind, amount, counterparty, category_id, note,
              happened_at, payment_method, status, raw_data, created_at, updated_at, device_id, version, deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?14, ?15, 1, 0)",
            params![
                id,
                input.account_id,
                input.source.as_str(),
                input.external_id,
                input.kind.as_str(),
                input.amount,
                input.counterparty,
                input.category_id,
                input.note,
                input.happened_at,
                input.payment_method,
                input.status,
                input.raw_data,
                timestamp,
                self.device_id,
            ],
        )?;
        record_change(
            &tx,
            EntityType::Transaction,
            &id,
            SyncOp::Insert,
            None,
            &timestamp,
            self.device_id,
        )?;
        tx.commit()?;

        get_transaction(self.conn, &id, false)?.ok_or_else(|| AppError::NotFound(id))
    }

    fn get_transaction(&self, id: &str) -> AppResult<Option<Transaction>> {
        get_transaction(self.conn, id, false)
    }

    fn find_by_external(
        &self,
        source: TransactionSource,
        external_id: &str,
    ) -> AppResult<Option<Transaction>> {
        Ok(self
            .conn
            .query_row(
                &format!(
                    "SELECT {TRANSACTION_COLUMNS} FROM transactions
                     WHERE source = ?1 AND external_id = ?2 AND deleted = 0"
                ),
                params![source.as_str(), external_id],
                map_transaction,
            )
            .optional()?)
    }

    fn list_transactions(&self, include_deleted: bool) -> AppResult<Vec<Transaction>> {
        let sql = format!(
            "SELECT {TRANSACTION_COLUMNS} FROM transactions
             {} ORDER BY happened_at DESC, id DESC",
            if include_deleted {
                ""
            } else {
                "WHERE deleted = 0"
            }
        );
        let mut statement = self.conn.prepare(&sql)?;
        let rows = statement.query_map([], map_transaction)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn update_transaction(
        &self,
        id: &str,
        patch: &TransactionPatch,
    ) -> AppResult<Option<Transaction>> {
        let mut sets: Vec<String> = Vec::new();
        let mut values: Vec<Box<dyn ToSql>> = Vec::new();
        let mut set = |column: &str, value: Box<dyn ToSql>| {
            sets.push(format!("{column} = ?"));
            values.push(value);
        };

        if let Some(value) = &patch.account_id {
            set("account_id", Box::new(value.clone()));
        }
        if let Some(value) = patch.kind {
            set("kind", Box::new(value.as_str().to_string()));
        }
        if let Some(value) = patch.amount {
            set("amount", Box::new(value));
        }
        if let Some(value) = &patch.counterparty {
            set("counterparty", Box::new(value.clone()));
        }
        if let Some(value) = &patch.category_id {
            set("category_id", Box::new(value.clone()));
        }
        if let Some(value) = &patch.note {
            set("note", Box::new(value.clone()));
        }
        if let Some(value) = &patch.happened_at {
            set("happened_at", Box::new(value.clone()));
        }
        if let Some(value) = &patch.payment_method {
            set("payment_method", Box::new(value.clone()));
        }
        if let Some(value) = &patch.status {
            set("status", Box::new(value.clone()));
        }
        if let Some(value) = &patch.raw_data {
            set("raw_data", Box::new(value.clone()));
        }

        let timestamp = now();
        set("updated_at", Box::new(timestamp.clone()));
        set("device_id", Box::new(self.device_id.to_string()));
        sets.push("version = version + 1".to_string());
        values.push(Box::new(id.to_string()));

        let sql = format!(
            "UPDATE transactions SET {} WHERE id = ? AND deleted = 0",
            sets.join(", ")
        );

        let tx = self.conn.unchecked_transaction()?;
        let affected = tx.execute(&sql, params_from_iter(values.iter().map(|value| &**value)))?;
        if affected == 0 {
            return Ok(None);
        }

        let updated =
            get_transaction(&tx, id, false)?.ok_or_else(|| AppError::NotFound(id.to_string()))?;
        let payload = serialize_payload(&updated)?;
        record_change(
            &tx,
            EntityType::Transaction,
            id,
            SyncOp::Update,
            Some(&payload),
            &timestamp,
            self.device_id,
        )?;
        tx.commit()?;

        Ok(Some(updated))
    }

    fn soft_delete_transaction(&self, id: &str) -> AppResult<bool> {
        let timestamp = now();
        let tx = self.conn.unchecked_transaction()?;
        let affected = tx.execute(
            "UPDATE transactions
             SET deleted = 1, updated_at = ?2, device_id = ?3, version = version + 1
             WHERE id = ?1 AND deleted = 0",
            params![id, timestamp, self.device_id],
        )?;
        if affected == 0 {
            return Ok(false);
        }
        record_change(
            &tx,
            EntityType::Transaction,
            id,
            SyncOp::Delete,
            None,
            &timestamp,
            self.device_id,
        )?;
        tx.commit()?;
        Ok(true)
    }
}

impl CategoryRepository for SqliteRepository<'_> {
    fn insert_category(&self, input: &NewCategory) -> AppResult<Category> {
        if input.name.trim().is_empty() {
            return Err(AppError::Invalid("分类名称不能为空".to_string()));
        }
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO categories
             (id, name, parent_id, icon, color, sort_order, created_at, updated_at, device_id, version, deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8, 1, 0)",
            params![
                id,
                input.name,
                input.parent_id,
                input.icon,
                input.color,
                input.sort_order,
                timestamp,
                self.device_id,
            ],
        )?;
        record_change(
            &tx,
            EntityType::Category,
            &id,
            SyncOp::Insert,
            None,
            &timestamp,
            self.device_id,
        )?;
        tx.commit()?;

        get_category(self.conn, &id, false)?.ok_or_else(|| AppError::NotFound(id))
    }

    fn get_category(&self, id: &str) -> AppResult<Option<Category>> {
        get_category(self.conn, id, false)
    }

    fn list_categories(&self, include_deleted: bool) -> AppResult<Vec<Category>> {
        let sql = format!(
            "SELECT {CATEGORY_COLUMNS} FROM categories
             {} ORDER BY sort_order ASC, name ASC",
            if include_deleted {
                ""
            } else {
                "WHERE deleted = 0"
            }
        );
        let mut statement = self.conn.prepare(&sql)?;
        let rows = statement.query_map([], map_category)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn update_category(&self, id: &str, patch: &CategoryPatch) -> AppResult<Option<Category>> {
        let mut sets: Vec<String> = Vec::new();
        let mut values: Vec<Box<dyn ToSql>> = Vec::new();
        let mut set = |column: &str, value: Box<dyn ToSql>| {
            sets.push(format!("{column} = ?"));
            values.push(value);
        };

        if let Some(value) = &patch.name {
            if value.trim().is_empty() {
                return Err(AppError::Invalid("分类名称不能为空".to_string()));
            }
            set("name", Box::new(value.clone()));
        }
        if let Some(value) = &patch.parent_id {
            set("parent_id", Box::new(value.clone()));
        }
        if let Some(value) = &patch.icon {
            set("icon", Box::new(value.clone()));
        }
        if let Some(value) = &patch.color {
            set("color", Box::new(value.clone()));
        }
        if let Some(value) = patch.sort_order {
            set("sort_order", Box::new(value));
        }

        let timestamp = now();
        set("updated_at", Box::new(timestamp.clone()));
        set("device_id", Box::new(self.device_id.to_string()));
        sets.push("version = version + 1".to_string());
        values.push(Box::new(id.to_string()));

        let sql = format!(
            "UPDATE categories SET {} WHERE id = ? AND deleted = 0",
            sets.join(", ")
        );
        let tx = self.conn.unchecked_transaction()?;
        let affected = tx.execute(&sql, params_from_iter(values.iter().map(|value| &**value)))?;
        if affected == 0 {
            return Ok(None);
        }

        let updated =
            get_category(&tx, id, false)?.ok_or_else(|| AppError::NotFound(id.to_string()))?;
        let payload = serialize_payload(&updated)?;
        record_change(
            &tx,
            EntityType::Category,
            id,
            SyncOp::Update,
            Some(&payload),
            &timestamp,
            self.device_id,
        )?;
        tx.commit()?;

        Ok(Some(updated))
    }

    fn soft_delete_category(&self, id: &str) -> AppResult<bool> {
        let timestamp = now();
        let tx = self.conn.unchecked_transaction()?;
        let affected = tx.execute(
            "UPDATE categories
             SET deleted = 1, updated_at = ?2, device_id = ?3, version = version + 1
             WHERE id = ?1 AND deleted = 0",
            params![id, timestamp, self.device_id],
        )?;
        if affected == 0 {
            return Ok(false);
        }
        record_change(
            &tx,
            EntityType::Category,
            id,
            SyncOp::Delete,
            None,
            &timestamp,
            self.device_id,
        )?;
        tx.commit()?;
        Ok(true)
    }
}

impl AccountRepository for SqliteRepository<'_> {
    fn list_accounts(&self, include_deleted: bool) -> AppResult<Vec<Account>> {
        let sql = format!(
            "SELECT id, name, type, created_at, updated_at, device_id, version, deleted
             FROM accounts {} ORDER BY created_at ASC",
            if include_deleted {
                ""
            } else {
                "WHERE deleted = 0"
            }
        );
        let mut statement = self.conn.prepare(&sql)?;
        let rows = statement.query_map([], |row| {
            Ok(Account {
                id: row.get("id")?,
                name: row.get("name")?,
                kind: row.get("type")?,
                created_at: row.get("created_at")?,
                updated_at: row.get("updated_at")?,
                device_id: row.get("device_id")?,
                version: row.get("version")?,
                deleted: bit(row.get("deleted")?),
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }
}

impl SyncChangeRepository for SqliteRepository<'_> {
    fn list_since(&self, since_seq: i64, limit: i64) -> AppResult<Vec<SyncChange>> {
        let limit = limit.clamp(1, 10_000);
        let mut statement = self.conn.prepare(
            "SELECT seq, entity_type, entity_id, op, payload, created_at, device_id
             FROM sync_changes WHERE seq > ?1 ORDER BY seq ASC LIMIT ?2",
        )?;
        let rows = statement.query_map(params![since_seq, limit], map_sync_change)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn count_since(&self, since_seq: i64) -> AppResult<i64> {
        Ok(self.conn.query_row(
            "SELECT COUNT(*) FROM sync_changes WHERE seq > ?1",
            [since_seq],
            |row| row.get(0),
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migrate, open_in_memory};

    fn repository(conn: &Connection) -> SqliteRepository<'_> {
        SqliteRepository::new(conn, "test-device")
    }

    fn sample(external_id: &str) -> NewTransaction {
        NewTransaction {
            external_id: external_id.to_string(),
            amount: 1234,
            counterparty: Some("测试商家".to_string()),
            happened_at: "2026-11-19T10:00:00.000Z".to_string(),
            ..Default::default()
        }
    }

    fn setup() -> Connection {
        let conn = open_in_memory().unwrap();
        migrate(&conn).unwrap();
        conn
    }

    #[test]
    fn transaction_insert_and_find_by_external() {
        let conn = setup();
        let repo = repository(&conn);
        let created = repo.insert_transaction(&sample("TX-1")).unwrap();

        assert_eq!(created.amount, 1234);
        assert_eq!(created.kind, TransactionKind::Expense);
        assert_eq!(created.source, TransactionSource::Manual);
        assert_eq!(created.device_id, "test-device");
        assert_eq!(created.version, 1);
        assert!(!created.deleted);

        let found = repo
            .find_by_external(TransactionSource::Manual, "TX-1")
            .unwrap()
            .unwrap();
        assert_eq!(found.id, created.id);
    }

    #[test]
    fn duplicate_source_and_external_id_is_rejected() {
        let conn = setup();
        let repo = repository(&conn);
        repo.insert_transaction(&sample("DUP-1")).unwrap();
        assert!(repo.insert_transaction(&sample("DUP-1")).is_err());
    }

    #[test]
    fn update_supports_partial_and_nullable_fields() {
        let conn = setup();
        let repo = repository(&conn);
        let created = repo.insert_transaction(&sample("TX-2")).unwrap();

        let patch = TransactionPatch {
            amount: Some(2000),
            note: Some(Some("备注".to_string())),
            ..Default::default()
        };
        let updated = repo
            .update_transaction(&created.id, &patch)
            .unwrap()
            .unwrap();
        assert_eq!(updated.amount, 2000);
        assert_eq!(updated.note.as_deref(), Some("备注"));
        assert_eq!(updated.version, 2);

        let cleared = repo
            .update_transaction(
                &created.id,
                &TransactionPatch {
                    note: Some(None),
                    ..Default::default()
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(cleared.note, None);
        assert_eq!(cleared.version, 3);
    }

    #[test]
    fn delete_is_soft_and_hides_transaction() {
        let conn = setup();
        let repo = repository(&conn);
        let created = repo.insert_transaction(&sample("TX-3")).unwrap();

        assert!(repo.soft_delete_transaction(&created.id).unwrap());
        assert!(!repo.soft_delete_transaction(&created.id).unwrap());
        assert!(repo.get_transaction(&created.id).unwrap().is_none());
        assert!(repo.list_transactions(false).unwrap().is_empty());
        assert_eq!(repo.list_transactions(true).unwrap().len(), 1);
        assert!(repo.list_transactions(true).unwrap()[0].deleted);
    }

    #[test]
    fn category_crud() {
        let conn = setup();
        let repo = repository(&conn);
        let created = repo
            .insert_category(&NewCategory {
                name: "娱乐".to_string(),
                sort_order: 50,
                ..Default::default()
            })
            .unwrap();
        assert!(repo.get_category(&created.id).unwrap().is_some());

        let updated = repo
            .update_category(
                &created.id,
                &CategoryPatch {
                    name: Some("休闲娱乐".to_string()),
                    ..Default::default()
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(updated.name, "休闲娱乐");
        assert_eq!(updated.version, 2);

        assert!(repo.soft_delete_category(&created.id).unwrap());
        assert!(repo.get_category(&created.id).unwrap().is_none());
    }

    #[test]
    fn every_write_appends_sync_change() {
        let conn = setup();
        let repo = repository(&conn);
        let created = repo.insert_transaction(&sample("TX-4")).unwrap();
        repo.update_transaction(
            &created.id,
            &TransactionPatch {
                amount: Some(99),
                ..Default::default()
            },
        )
        .unwrap();
        repo.soft_delete_transaction(&created.id).unwrap();

        let changes = repo.list_since(0, 100).unwrap();
        assert_eq!(changes.len(), 3);
        assert_eq!(changes[0].op, SyncOp::Insert);
        assert_eq!(changes[1].op, SyncOp::Update);
        assert_eq!(changes[2].op, SyncOp::Delete);
        assert!(changes[1].payload.is_some());
        assert!(changes
            .iter()
            .all(|change| change.device_id == "test-device"));
        assert_eq!(repo.count_since(0).unwrap(), 3);
    }
}
