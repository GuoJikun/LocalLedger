# 净账（LocalLedger）

本地运行、离线优先的记账软件。通过导入支付宝 / 微信账单生成流水与统计。

- **数据只留在自己电脑**：无账号、无埋点、默认完全离线。
- **技术栈**：Tauri 2 + Vue 3 + TypeScript + Rust + SQLite。
- **平台**：Windows / macOS / Linux 同源。
- **当前阶段**：M3 流水与分类（详见 [`docs/计划.md`](docs/计划.md)）。

## 开发环境

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

CI（`.github/workflows/ci.yml`）会在 Windows / macOS / Linux 三个平台执行上述检查与编译。

## 目录结构

```
src-tauri/    Rust 后端：命令、领域逻辑、导入解析、Repository、数据库
src/          Vue 前端：组件、页面、状态
docs/         规划、执行计划与变更记录
scripts/      图标源图生成脚本
```

设计细节见 [`docs/规划.md`](docs/规划.md)，任务拆解见 [`docs/计划.md`](docs/计划.md)。
