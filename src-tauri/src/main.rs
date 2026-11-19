// 发布版本在 Windows 上隐藏控制台窗口。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    local_ledger_lib::run()
}
