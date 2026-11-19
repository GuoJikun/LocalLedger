import { ref, watch } from "vue";
import { getCurrentWindow, type Theme } from "@tauri-apps/api/window";

export type ThemeMode = "light" | "dark" | "system";

const STORAGE_KEY = "local-ledger.theme-mode";
const VALID_MODES: ThemeMode[] = ["light", "dark", "system"];

const media = window.matchMedia("(prefers-color-scheme: dark)");

function readStoredMode(): ThemeMode {
  try {
    const value = localStorage.getItem(STORAGE_KEY) as ThemeMode | null;
    return value && VALID_MODES.includes(value) ? value : "system";
  } catch {
    return "system";
  }
}

const mode = ref<ThemeMode>(readStoredMode());
const isDark = ref(false);

function syncNativeTitleBar() {
  // 直接检测 Tauri IPC 注入对象，避免浏览器预览模式误判。
  if (!("__TAURI_INTERNALS__" in window)) return;
  // system 模式传 null 交还系统；显式模式同步给原生标题栏（Windows 上生效）。
  const theme: Theme | null = mode.value === "system" ? null : mode.value;
  void getCurrentWindow()
    .setTheme(theme)
    .catch((error) => {
      // 平台不支持或缺少权限时记录，避免静默失败。
      console.warn("同步原生标题栏主题失败：", error);
    });
}

function apply() {
  const dark = mode.value === "dark" || (mode.value === "system" && media.matches);
  isDark.value = dark;
  document.documentElement.dataset.theme = dark ? "dark" : "light";
  // Element Plus 的深色主题挂载在 html.dark 上，必须与自定义变量同步切换。
  document.documentElement.classList.toggle("dark", dark);
  syncNativeTitleBar();
}

watch(
  mode,
  (value) => {
    try {
      localStorage.setItem(STORAGE_KEY, value);
    } catch {
      // 隐私模式下 localStorage 可能不可写，忽略即可。
    }
    apply();
  },
  { immediate: true },
);

media.addEventListener("change", apply);

export function useTheme() {
  function setMode(value: ThemeMode) {
    mode.value = value;
  }

  return { mode, isDark, setMode };
}
