import { fileURLToPath, URL } from "node:url";
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// Tauri 在 dev 模式下会设置 TAURI_DEV_HOST（移动端/远程调试需要）。
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  // Tauri 的 CLI 会解析 Vite 输出，清屏会影响日志阅读。
  clearScreen: false,
  build: {
    // 图表库拆分后仍约 550KB，高于默认 500KB 阈值，属预期。
    chunkSizeWarningLimit: 700,
    rollupOptions: {
      output: {
        // 图表库单独成块，避免主包过大。
        manualChunks(id) {
          if (id.includes("node_modules/echarts") || id.includes("node_modules/zrender")) {
            return "charts";
          }
        },
      },
    },
  },
  server: {
    port: 1430,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
