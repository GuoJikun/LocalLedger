mod commands;
mod db;
mod domain;
mod error;
mod importer;
mod repository;
mod state;

use std::sync::Mutex;

use tauri::{AppHandle, Manager, Runtime};

use crate::state::AppState;

/// 返回后端版本号，供前端「关于」区域展示。
#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 返回应用数据目录并确保其可写。
#[tauri::command]
fn app_data_dir<R: Runtime>(app: AppHandle<R>) -> Result<String, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.to_string_lossy().into_owned())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let database_path = data_dir.join("local-ledger.sqlite3");

            let conn = db::open(&database_path)?;
            db::migrate(&conn)?;
            let device = db::ensure_device(&conn)?;

            app.manage(AppState {
                conn: Mutex::new(conn),
                device_id: device.id.clone(),
                device_name: device.name.clone(),
                database_path: database_path.to_string_lossy().into_owned(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_version,
            app_data_dir,
            commands::data_overview,
            commands::device_info,
            commands::list_transactions,
            commands::get_transaction,
            commands::find_transaction_by_external,
            commands::create_transaction,
            commands::update_transaction,
            commands::delete_transaction,
            commands::list_accounts,
            commands::list_categories,
            commands::get_category,
            commands::create_category,
            commands::update_category,
            commands::delete_category,
            commands::list_sync_changes,
            commands::cursor_version,
            commands::preview_bill,
            commands::import_bill,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_version_matches_cargo_package_version() {
        assert_eq!(app_version(), env!("CARGO_PKG_VERSION"));
        assert!(!app_version().is_empty());
    }
}
