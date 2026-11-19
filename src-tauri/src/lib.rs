use tauri::{AppHandle, Manager, Runtime};

/// 返回后端版本号，供前端「关于」区域展示。
#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 返回应用数据目录并确保其可写。
///
/// 按规划，本地 SQLite 文件唯一数据源必须落在 `app_data_dir`，
/// 三端路径由 Tauri 统一解析，不硬编码平台路径。
#[tauri::command]
fn app_data_dir<R: Runtime>(app: AppHandle<R>) -> Result<String, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.to_string_lossy().into_owned())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![app_version, app_data_dir])
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
