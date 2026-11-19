import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

/** 读取 Rust 侧的版本号与数据目录，用于验证 Tauri IPC 已打通。 */
export function useAppInfo() {
  const version = ref("—");
  const dataDir = ref("—");
  const error = ref("");

  async function load() {
    try {
      version.value = await invoke<string>("app_version");
      dataDir.value = await invoke<string>("app_data_dir");
      error.value = "";
    } catch (e) {
      // 在纯浏览器 dev 环境（非 Tauri 窗口）下 invoke 不可用，展示占位即可。
      error.value = e instanceof Error ? e.message : String(e);
    }
  }

  onMounted(load);

  return { version, dataDir, error, load };
}
