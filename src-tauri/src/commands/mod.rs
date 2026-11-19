//! Tauri 命令层：只做参数传递与错误转换，业务逻辑在 repository。

use rusqlite::OptionalExtension;
use serde::Serialize;
use tauri::State;

use crate::domain::{
    Account, Category, CategoryPatch, CategoryRule, CategoryRulePatch, CategorySlice, DailyPoint,
    Device, MonthSummary, NewCategory, NewCategoryRule, NewTransaction, SyncChange, Transaction,
    TransactionFilter, TransactionKind, TransactionPage, TransactionPatch, TransactionSource,
};
use crate::error::AppError;
use crate::importer::{
    self,
    service::{self, ImportSummary},
};
use crate::repository::{
    AccountRepository, CategoryRepository, CategoryRuleRepository, SqliteRepository,
    StatisticsRepository, SyncChangeRepository, TransactionRepository,
};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct DataOverview {
    pub device_id: String,
    pub device_name: String,
    pub database_path: String,
    pub account_count: i64,
    pub transaction_count: i64,
    pub category_count: i64,
    pub pending_changes: i64,
}

fn count(conn: &rusqlite::Connection, table: &str, where_clause: &str) -> Result<i64, String> {
    // table / where_clause 均为本模块内的常量，不接收用户输入。
    let sql = format!("SELECT COUNT(*) FROM {table} {where_clause}");
    conn.query_row(&sql, [], |row| row.get(0))
        .map_err(|error| error.to_string())
}

fn repo<'a>(conn: &'a rusqlite::Connection, state: &'a AppState) -> SqliteRepository<'a> {
    SqliteRepository::new(conn, &state.device_id)
}

#[tauri::command]
pub fn data_overview(state: State<'_, AppState>) -> Result<DataOverview, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    Ok(DataOverview {
        device_id: state.device_id.clone(),
        device_name: state.device_name.clone(),
        database_path: state.database_path.clone(),
        account_count: count(&conn, "accounts", "WHERE deleted = 0")?,
        transaction_count: count(&conn, "transactions", "WHERE deleted = 0")?,
        category_count: count(&conn, "categories", "WHERE deleted = 0")?,
        pending_changes: repo(&conn, &state).count_since(0)?,
    })
}

#[tauri::command]
pub fn device_info(state: State<'_, AppState>) -> Result<Device, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    let last_seen_at: String = conn
        .query_row(
            "SELECT last_seen_at FROM devices WHERE id = ?1",
            [&state.device_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    Ok(Device {
        id: state.device_id.clone(),
        name: state.device_name.clone(),
        last_seen_at,
    })
}

#[tauri::command]
pub fn list_transactions(state: State<'_, AppState>) -> Result<Vec<Transaction>, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .list_transactions(false)
        .map_err(String::from)
}

#[tauri::command]
pub fn search_transactions(
    filter: TransactionFilter,
    state: State<'_, AppState>,
) -> Result<TransactionPage, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .search_transactions(&filter)
        .map_err(String::from)
}

#[tauri::command]
pub fn get_transaction(
    id: String,
    state: State<'_, AppState>,
) -> Result<Option<Transaction>, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .get_transaction(&id)
        .map_err(String::from)
}

#[tauri::command]
pub fn find_transaction_by_external(
    source: TransactionSource,
    external_id: String,
    state: State<'_, AppState>,
) -> Result<Option<Transaction>, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .find_by_external(source, &external_id)
        .map_err(String::from)
}

#[tauri::command]
pub fn create_transaction(
    input: NewTransaction,
    state: State<'_, AppState>,
) -> Result<Transaction, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .insert_transaction(&input)
        .map_err(String::from)
}

#[tauri::command]
pub fn update_transaction(
    id: String,
    patch: TransactionPatch,
    state: State<'_, AppState>,
) -> Result<Option<Transaction>, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .update_transaction(&id, &patch)
        .map_err(String::from)
}

#[tauri::command]
pub fn delete_transaction(id: String, state: State<'_, AppState>) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .soft_delete_transaction(&id)
        .map_err(String::from)
}

#[tauri::command]
pub fn list_accounts(state: State<'_, AppState>) -> Result<Vec<Account>, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .list_accounts(false)
        .map_err(String::from)
}

#[tauri::command]
pub fn list_categories(state: State<'_, AppState>) -> Result<Vec<Category>, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .list_categories(false)
        .map_err(String::from)
}

#[tauri::command]
pub fn get_category(id: String, state: State<'_, AppState>) -> Result<Option<Category>, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state).get_category(&id).map_err(String::from)
}

#[tauri::command]
pub fn create_category(input: NewCategory, state: State<'_, AppState>) -> Result<Category, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .insert_category(&input)
        .map_err(String::from)
}

#[tauri::command]
pub fn update_category(
    id: String,
    patch: CategoryPatch,
    state: State<'_, AppState>,
) -> Result<Option<Category>, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .update_category(&id, &patch)
        .map_err(String::from)
}

#[tauri::command]
pub fn delete_category(id: String, state: State<'_, AppState>) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .soft_delete_category(&id)
        .map_err(String::from)
}

#[tauri::command]
pub fn monthly_summary(
    months: i64,
    state: State<'_, AppState>,
) -> Result<Vec<MonthSummary>, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .monthly_summary(months)
        .map_err(String::from)
}

#[tauri::command]
pub fn category_breakdown(
    month: Option<String>,
    kind: TransactionKind,
    state: State<'_, AppState>,
) -> Result<Vec<CategorySlice>, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .category_breakdown(month.as_deref(), kind)
        .map_err(String::from)
}

#[tauri::command]
pub fn daily_trend(month: String, state: State<'_, AppState>) -> Result<Vec<DailyPoint>, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .daily_trend(&month)
        .map_err(String::from)
}

#[tauri::command]
pub fn list_rules(state: State<'_, AppState>) -> Result<Vec<CategoryRule>, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state).list_rules().map_err(String::from)
}

#[tauri::command]
pub fn create_rule(
    input: NewCategoryRule,
    state: State<'_, AppState>,
) -> Result<CategoryRule, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .insert_rule(&input)
        .map_err(String::from)
}

#[tauri::command]
pub fn update_rule(
    id: String,
    patch: CategoryRulePatch,
    state: State<'_, AppState>,
) -> Result<Option<CategoryRule>, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .update_rule(&id, &patch)
        .map_err(String::from)
}

#[tauri::command]
pub fn delete_rule(id: String, state: State<'_, AppState>) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .soft_delete_rule(&id)
        .map_err(String::from)
}

#[tauri::command]
pub fn list_sync_changes(
    since_seq: i64,
    limit: i64,
    state: State<'_, AppState>,
) -> Result<Vec<SyncChange>, String> {
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    repo(&conn, &state)
        .list_since(since_seq, limit)
        .map_err(String::from)
}

#[tauri::command]
pub fn preview_bill(path: String) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
    let text = importer::decode(&bytes).map_err(|error| error.to_string())?;
    let platform = importer::detect(&text).map_err(|error| error.to_string())?;
    Ok(platform.label().to_string())
}

fn import_bill_inner(path: &str, state: &AppState) -> Result<ImportSummary, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    let bill = importer::parse(&bytes).map_err(|error| error.to_string())?;
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    service::persist(&repo(&conn, state), bill).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_bill(path: String, state: State<'_, AppState>) -> Result<ImportSummary, String> {
    import_bill_inner(&path, &state)
}

#[tauri::command]
pub fn cursor_version(state: State<'_, AppState>) -> Result<i64, String> {
    // 供同步模块读取当前游标；M1 仅暴露基础能力。
    let conn = state.conn.lock().map_err(|_| AppError::LockPoisoned)?;
    let value: Option<i64> = conn
        .query_row("SELECT MAX(seq) FROM sync_changes", [], |row| row.get(0))
        .optional()
        .map_err(|error| error.to_string())?;
    Ok(value.unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::db::{migrate, open_in_memory};

    fn test_state() -> AppState {
        let conn = open_in_memory().unwrap();
        migrate(&conn).unwrap();
        AppState {
            conn: Mutex::new(conn),
            device_id: "test-device".to_string(),
            device_name: "测试机".to_string(),
            database_path: ":memory:".to_string(),
        }
    }

    #[test]
    fn import_command_reads_gbk_file_and_is_idempotent() {
        let state = test_state();
        let source = include_str!("../../tests/fixtures/alipay_sample.csv");
        let (gbk, _, _) = encoding_rs::GBK.encode(source);
        let path = std::env::temp_dir().join("local-ledger-import-command-test.csv");
        std::fs::write(&path, gbk.as_ref()).unwrap();

        let path = path.to_string_lossy().into_owned();
        let first = import_bill_inner(&path, &state).unwrap();
        assert_eq!(first.platform, "支付宝");
        assert_eq!(first.imported, 3);
        assert_eq!(first.classified, 3);

        let second = import_bill_inner(&path, &state).unwrap();
        assert_eq!(second.imported, 0);
        assert_eq!(second.duplicates, 3);

        std::fs::remove_file(path).unwrap();
    }
}
