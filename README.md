# 净账（LocalLedger）

本地运行、离线优先的记账软件。通过导入支付宝 / 微信账单生成流水与统计。

- **数据只留在自己电脑**：无账号、无埋点、默认完全离线。
- **技术栈**：Tauri 2 + Vue 3 + TypeScript + Rust + SQLite。
- **平台**：Windows / macOS / Linux 同源。
- **当前状态**：MVP 已完成（M0~M5），局域网同步（M6）为后续版本。

## 功能

- 导入支付宝（GBK）/ 微信（UTF-8 BOM）CSV 账单，自动识别编码与表头、跳过说明行、精确解析金额（整数「分」）。
- 按 `source + external_id` 去重，重复导入不会产生脏数据；错误行返回明确行号与原因。
- 交易流水：搜索、按时间/类型/分类筛选、分页、手工录入、编辑、软删除。
- 分类层级与自动分类规则（关键词 + 匹配字段 + 优先级）。
- 统计：月度收支、分类占比、每日趋势（ECharts）。
- 数据库备份/恢复、流水 CSV 导出。
- 浅色 / 深色 / 跟随系统主题。

## 使用

1. 打开应用后进入「导入账单」，选择或拖入账单 CSV，确认识别的平台后点击「开始导入」。
2. 在「交易流水」中筛选、编辑或手工补记；在「分类管理」中维护分类与自动分类规则。
3. 在「统计」查看图表；在「设置」中备份数据库、恢复备份或导出 CSV。

数据文件位置：

- Windows：`%APPDATA%\com.localledger.desktop\local-ledger.sqlite3`
- macOS：`~/Library/Application Support/com.localledger.desktop/local-ledger.sqlite3`
- Linux：`~/.local/share/com.localledger.desktop/local-ledger.sqlite3`

## 开发

需要 Node.js、pnpm、Rust 工具链，以及 [Tauri 2 的系统依赖](https://v2.tauri.app/start/prerequisites/)。

```bash
pnpm install        # 安装前端依赖
pnpm tauri dev      # 启动开发窗口（前端热更新 + Rust 后端）
pnpm build          # 仅构建前端
pnpm tauri build    # 构建并打包当前平台安装包
```

Rust 侧检查：

```bash
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## 打包

平台配置在 `src-tauri/tauri.*.conf.json`：

| 平台 | 目标 | 产物目录 |
|---|---|---|
| Windows | NSIS | `src-tauri/target/release/bundle/nsis` |
| macOS | dmg、app | `src-tauri/target/release/bundle/dmg`、`.../macos` |
| Linux | deb、rpm、AppImage | `src-tauri/target/release/bundle/deb`、`.../rpm`、`.../appimage` |

三端同发由 `.github/workflows/release.yml` 在推送 `v*` 标签时触发，产物上传为 CI artifact。macOS 签名与公证需要开发者账号，配置 `APPLE_SIGNING_IDENTITY` / `APPLE_NOTARIZE_*` 后在 macOS 构建机上启用；未配置时产出未签名包。Windows 当前只打包 NSIS（不依赖 WiX Toolset）。

CI（`.github/workflows/ci.yml`）在 Windows / macOS / Linux 三平台执行格式化、clippy、测试、前端构建与 Tauri 编译。

## 目录结构

```
src-tauri/    Rust 后端：命令、领域模型、导入解析、Repository、数据库
src/          Vue 前端：组件、页面、API 封装
docs/         规划、执行计划与变更记录
scripts/      图标源图生成脚本
```

设计细节见 [`docs/规划.md`](docs/规划.md)，任务拆解见 [`docs/计划.md`](docs/计划.md)，变更历史见 [`docs/变更记录.md`](docs/变更记录.md)。
