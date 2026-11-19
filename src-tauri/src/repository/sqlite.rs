//! Repository 的 SQLite 实现：所有写操作都在事务内追加 `sync_changes`。

use chrono::SecondsFormat;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row, ToSql};
use serde::Serialize;
use uuid::Uuid;

use crate::domain::{
    now, Account, Category, CategoryPatch, CategoryRule, CategoryRulePatch, CategorySlice,
    DailyPoint, EntityType, MonthSummary, NewCategory, NewCategoryRule, NewTransaction, SyncChange,
    SyncOp, Transaction, TransactionFilter, TransactionKind, TransactionPage, TransactionPatch,
    TransactionSource,
};
use crate::error::{AppError, AppResult};

use super::{
    AccountRepository, CategoryRepository, CategoryRuleRepository, StatisticsRepository,
    SyncChangeRepository, TransactionRepository,
};

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

fn map_category_rule(row: &Row<'_>) -> rusqlite::Result<CategoryRule> {
    Ok(CategoryRule {
        id: row.get("id")?,
        keyword: row.get("keyword")?,
        match_field: row.get("match_field")?,
        category_id: row.get("category_id")?,
        priority: row.get("priority")?,
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

/// 把筛选条件编译成参数化 SQL，杜绝字符串拼接用户输入。
fn transaction_filter_sql(filter: &TransactionFilter) -> (String, Vec<Box<dyn ToSql>>) {
    let mut clauses = vec!["deleted = 0".to_string()];
    let mut values: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(keyword) = &filter.keyword {
        let pattern = format!("%{}%", keyword.trim().to_lowercase());
        clauses.push(
            "(LOWER(COALESCE(counterparty, '')) LIKE ?
              OR LOWER(COALESCE(note, '')) LIKE ?
              OR LOWER(external_id) LIKE ?)"
                .to_string(),
        );
        values.push(Box::new(pattern.clone()));
        values.push(Box::new(pattern.clone()));
        values.push(Box::new(pattern));
    }
    if let Some(kind) = filter.kind {
        clauses.push("kind = ?".to_string());
        values.push(Box::new(kind.as_str().to_string()));
    }
    if let Some(category_id) = &filter.category_id {
        if category_id == "__none__" {
            clauses.push("category_id IS NULL".to_string());
        } else {
            clauses.push("category_id = ?".to_string());
            values.push(Box::new(category_id.clone()));
        }
    }
    if let Some(from) = &filter.from {
        clauses.push("happened_at >= ?".to_string());
        values.push(Box::new(from.clone()));
    }
    if let Some(to) = &filter.to {
        clauses.push("happened_at <= ?".to_string());
        values.push(Box::new(to.clone()));
    }
    if let Some(min) = filter.min_amount {
        clauses.push("amount >= ?".to_string());
        values.push(Box::new(min));
    }
    if let Some(max) = filter.max_amount {
        clauses.push("amount <= ?".to_string());
        values.push(Box::new(max));
    }

    (clauses.join(" AND "), values)
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

    fn search_transactions(&self, filter: &TransactionFilter) -> AppResult<TransactionPage> {
        let (where_sql, mut values) = transaction_filter_sql(filter);
        let limit = filter.limit.clamp(1, 500);
        let offset = filter.offset.max(0);

        let sql = format!(
            "SELECT {TRANSACTION_COLUMNS} FROM transactions WHERE {where_sql}
             ORDER BY happened_at DESC, id DESC LIMIT ? OFFSET ?"
        );
        values.push(Box::new(limit));
        values.push(Box::new(offset));

        let mut statement = self.conn.prepare(&sql)?;
        let rows = statement.query_map(
            params_from_iter(values.iter().map(|value| &**value)),
            map_transaction,
        )?;
        let items = rows.collect::<rusqlite::Result<Vec<_>>>()?;

        let count_sql = format!("SELECT COUNT(*) FROM transactions WHERE {where_sql}");
        let filter_values = &values[..values.len() - 2];
        let total: i64 = self.conn.query_row(
            &count_sql,
            params_from_iter(filter_values.iter().map(|value| &**value)),
            |row| row.get(0),
        )?;

        Ok(TransactionPage { items, total })
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

impl CategoryRuleRepository for SqliteRepository<'_> {
    fn list_rules(&self) -> AppResult<Vec<CategoryRule>> {
        let mut statement = self.conn.prepare(
            "SELECT id, keyword, match_field, category_id, priority, created_at, updated_at,
                    device_id, version, deleted
             FROM category_rules
             WHERE deleted = 0
             ORDER BY priority DESC, id ASC",
        )?;
        let rows = statement.query_map([], map_category_rule)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn insert_rule(&self, input: &NewCategoryRule) -> AppResult<CategoryRule> {
        if input.keyword.trim().is_empty() {
            return Err(AppError::Invalid("规则关键词不能为空".to_string()));
        }
        let id = Uuid::new_v4().to_string();
        let timestamp = now();
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO category_rules
             (id, keyword, match_field, category_id, priority, created_at, updated_at,
              device_id, version, deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, 1, 0)",
            params![
                id,
                input.keyword,
                input.match_field,
                input.category_id,
                input.priority,
                timestamp,
                self.device_id,
            ],
        )?;
        record_change(
            &tx,
            EntityType::CategoryRule,
            &id,
            SyncOp::Insert,
            None,
            &timestamp,
            self.device_id,
        )?;
        tx.commit()?;

        self.conn
            .query_row(
                "SELECT id, keyword, match_field, category_id, priority, created_at, updated_at,
                        device_id, version, deleted
                 FROM category_rules WHERE id = ?1 AND deleted = 0",
                [&id],
                map_category_rule,
            )
            .map_err(Into::into)
    }

    fn update_rule(&self, id: &str, patch: &CategoryRulePatch) -> AppResult<Option<CategoryRule>> {
        let mut sets: Vec<String> = Vec::new();
        let mut values: Vec<Box<dyn ToSql>> = Vec::new();
        let mut set = |column: &str, value: Box<dyn ToSql>| {
            sets.push(format!("{column} = ?"));
            values.push(value);
        };

        if let Some(value) = &patch.keyword {
            if value.trim().is_empty() {
                return Err(AppError::Invalid("规则关键词不能为空".to_string()));
            }
            set("keyword", Box::new(value.clone()));
        }
        if let Some(value) = &patch.match_field {
            set("match_field", Box::new(value.clone()));
        }
        if let Some(value) = &patch.category_id {
            set("category_id", Box::new(value.clone()));
        }
        if let Some(value) = patch.priority {
            set("priority", Box::new(value));
        }

        let timestamp = now();
        set("updated_at", Box::new(timestamp.clone()));
        set("device_id", Box::new(self.device_id.to_string()));
        sets.push("version = version + 1".to_string());
        values.push(Box::new(id.to_string()));

        let sql = format!(
            "UPDATE category_rules SET {} WHERE id = ? AND deleted = 0",
            sets.join(", ")
        );
        let tx = self.conn.unchecked_transaction()?;
        let affected = tx.execute(&sql, params_from_iter(values.iter().map(|value| &**value)))?;
        if affected == 0 {
            return Ok(None);
        }

        let updated = tx.query_row(
            "SELECT id, keyword, match_field, category_id, priority, created_at, updated_at,
                    device_id, version, deleted
             FROM category_rules WHERE id = ?1 AND deleted = 0",
            [id],
            map_category_rule,
        )?;
        let payload = serialize_payload(&updated)?;
        record_change(
            &tx,
            EntityType::CategoryRule,
            id,
            SyncOp::Update,
            Some(&payload),
            &timestamp,
            self.device_id,
        )?;
        tx.commit()?;

        Ok(Some(updated))
    }

    fn soft_delete_rule(&self, id: &str) -> AppResult<bool> {
        let timestamp = now();
        let tx = self.conn.unchecked_transaction()?;
        let affected = tx.execute(
            "UPDATE category_rules
             SET deleted = 1, updated_at = ?2, device_id = ?3, version = version + 1
             WHERE id = ?1 AND deleted = 0",
            params![id, timestamp, self.device_id],
        )?;
        if affected == 0 {
            return Ok(false);
        }
        record_change(
            &tx,
            EntityType::CategoryRule,
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

impl StatisticsRepository for SqliteRepository<'_> {
    fn monthly_summary(&self, months: i64) -> AppResult<Vec<MonthSummary>> {
        let months = months.clamp(1, 120);
        let since = chrono::Utc::now()
            .checked_sub_months(chrono::Months::new(months as u32))
            .unwrap_or_else(chrono::Utc::now)
            .to_rfc3339_opts(SecondsFormat::Millis, true);

        let mut statement = self.conn.prepare(
            "SELECT substr(happened_at, 1, 7) AS month,
                    COALESCE(SUM(CASE WHEN kind = 'income' THEN amount ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN kind = 'expense' THEN amount ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN kind = 'transfer' THEN amount ELSE 0 END), 0),
                    COUNT(*)
             FROM transactions
             WHERE deleted = 0 AND happened_at >= ?1
             GROUP BY month
             ORDER BY month DESC",
        )?;
        let rows = statement.query_map([since], |row| {
            Ok(MonthSummary {
                month: row.get(0)?,
                income: row.get(1)?,
                expense: row.get(2)?,
                transfer: row.get(3)?,
                count: row.get(4)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(Into::into)
    }

    fn category_breakdown(
        &self,
        month: Option<&str>,
        kind: TransactionKind,
    ) -> AppResult<Vec<CategorySlice>> {
        let mut statement = self.conn.prepare(
            "SELECT COALESCE(t.category_id, '__none__') AS category_id,
                    COALESCE(c.name, '未分类') AS name,
                    COALESCE(SUM(t.amount), 0) AS total,
                    COUNT(*) AS cnt
             FROM transactions t
             LEFT JOIN categories c ON c.id = t.category_id
             WHERE t.deleted = 0 AND t.kind = ?1
                   AND (?2 IS NULL OR substr(t.happened_at, 1, 7) = ?2)
             GROUP BY category_id
             ORDER BY total DESC",
        )?;
        let rows = statement.query_map(params![kind.as_str(), month], |row| {
            Ok(CategorySlice {
                category_id: row.get(0)?,
                name: row.get(1)?,
                amount: row.get(2)?,
                count: row.get(3)?,
                percent: 0.0,
            })
        })?;
        let mut slices = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        let total: i64 = slices.iter().map(|slice| slice.amount).sum();
        if total > 0 {
            for slice in &mut slices {
                slice.percent = slice.amount as f64 / total as f64 * 100.0;
            }
        }
        Ok(slices)
    }

    fn daily_trend(&self, month: &str) -> AppResult<Vec<DailyPoint>> {
        let pattern = format!("{month}%");
        let mut statement = self.conn.prepare(
            "SELECT substr(happened_at, 1, 10) AS day,
                    COALESCE(SUM(CASE WHEN kind = 'income' THEN amount ELSE 0 END), 0),
                    COALESCE(SUM(CASE WHEN kind = 'expense' THEN amount ELSE 0 END), 0)
             FROM transactions
             WHERE deleted = 0 AND happened_at LIKE ?1
             GROUP BY day
             ORDER BY day ASC",
        )?;
        let rows = statement.query_map([pattern], |row| {
            Ok(DailyPoint {
                day: row.get(0)?,
                income: row.get(1)?,
                expense: row.get(2)?,
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

    fn make(
        external_id: &str,
        kind: TransactionKind,
        amount: i64,
        counterparty: &str,
        category_id: Option<String>,
    ) -> NewTransaction {
        NewTransaction {
            external_id: external_id.to_string(),
            kind,
            amount,
            counterparty: Some(counterparty.to_string()),
            category_id,
            happened_at: "2026-01-05T04:00:00.000Z".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn search_filters_and_paginates() {
        let conn = setup();
        let repo = repository(&conn);
        repo.insert_transaction(&make(
            "A",
            TransactionKind::Expense,
            1000,
            "星巴克",
            Some("cat_food".to_string()),
        ))
        .unwrap();
        repo.insert_transaction(&make("B", TransactionKind::Income, 5000, "某公司", None))
            .unwrap();
        repo.insert_transaction(&make(
            "C",
            TransactionKind::Expense,
            2000,
            "滴滴出行",
            Some("cat_transport".to_string()),
        ))
        .unwrap();

        assert_eq!(
            repo.search_transactions(&TransactionFilter {
                keyword: Some("滴滴".to_string()),
                ..Default::default()
            })
            .unwrap()
            .total,
            1
        );
        assert_eq!(
            repo.search_transactions(&TransactionFilter {
                kind: Some(TransactionKind::Income),
                ..Default::default()
            })
            .unwrap()
            .total,
            1
        );
        assert_eq!(
            repo.search_transactions(&TransactionFilter {
                category_id: Some("cat_food".to_string()),
                ..Default::default()
            })
            .unwrap()
            .total,
            1
        );
        assert_eq!(
            repo.search_transactions(&TransactionFilter {
                category_id: Some("__none__".to_string()),
                ..Default::default()
            })
            .unwrap()
            .total,
            1
        );
        assert_eq!(
            repo.search_transactions(&TransactionFilter {
                min_amount: Some(1500),
                ..Default::default()
            })
            .unwrap()
            .total,
            2
        );

        let page = repo
            .search_transactions(&TransactionFilter {
                limit: 1,
                offset: 1,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(page.total, 3);
        assert_eq!(page.items.len(), 1);
    }

    #[test]
    fn statistics_match_transactions() {
        let conn = setup();
        let repo = repository(&conn);
        let make_at =
            |id: &str, kind, amount, time: &str, category: Option<String>| NewTransaction {
                external_id: id.to_string(),
                kind,
                amount,
                counterparty: Some("测试".to_string()),
                category_id: category,
                happened_at: time.to_string(),
                ..Default::default()
            };

        repo.insert_transaction(&make_at(
            "S1",
            TransactionKind::Income,
            10_000,
            "2026-11-05T04:00:00.000Z",
            Some("cat_salary".to_string()),
        ))
        .unwrap();
        repo.insert_transaction(&make_at(
            "S2",
            TransactionKind::Expense,
            3_000,
            "2026-11-06T04:00:00.000Z",
            Some("cat_food".to_string()),
        ))
        .unwrap();
        repo.insert_transaction(&make_at(
            "S3",
            TransactionKind::Expense,
            2_000,
            "2026-10-05T04:00:00.000Z",
            Some("cat_transport".to_string()),
        ))
        .unwrap();
        repo.insert_transaction(&make_at(
            "S4",
            TransactionKind::Income,
            5_000,
            "2026-11-07T04:00:00.000Z",
            Some("cat_salary".to_string()),
        ))
        .unwrap();

        let months = repo.monthly_summary(12).unwrap();
        let november = months.iter().find(|item| item.month == "2026-11").unwrap();
        assert_eq!(november.income, 15_000);
        assert_eq!(november.expense, 3_000);
        assert_eq!(november.count, 3);
        assert!(months
            .iter()
            .any(|item| item.month == "2026-10" && item.expense == 2_000));

        let slices = repo
            .category_breakdown(Some("2026-11"), TransactionKind::Income)
            .unwrap();
        assert_eq!(slices.len(), 1);
        assert_eq!(slices[0].name, "工资");
        assert_eq!(slices[0].amount, 15_000);
        assert!((slices[0].percent - 100.0).abs() < 0.01);

        let trend = repo.daily_trend("2026-11").unwrap();
        assert_eq!(trend.len(), 3);
        assert_eq!(trend[0].day, "2026-11-05");
    }

    #[test]
    fn category_rule_crud() {
        let conn = setup();
        let repo = repository(&conn);
        let initial = repo.list_rules().unwrap().len();
        assert!(initial > 0);

        let created = repo
            .insert_rule(&NewCategoryRule {
                keyword: "咖啡".to_string(),
                match_field: "counterparty".to_string(),
                category_id: "cat_food".to_string(),
                priority: 999,
            })
            .unwrap();
        assert_eq!(created.priority, 999);

        let updated = repo
            .update_rule(
                &created.id,
                &CategoryRulePatch {
                    priority: Some(1000),
                    ..Default::default()
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(updated.priority, 1000);
        assert_eq!(updated.version, 2);

        assert!(repo.soft_delete_rule(&created.id).unwrap());
        assert_eq!(repo.list_rules().unwrap().len(), initial);
    }
}
