use std::sync::Mutex;

use rusqlite::Connection;

/// 应用运行时状态：数据库连接 + 当前设备身份。
pub struct AppState {
    pub conn: Mutex<Connection>,
    pub device_id: String,
    pub device_name: String,
    pub database_path: String,
}
