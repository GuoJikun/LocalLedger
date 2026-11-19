import { ref, watch } from "vue";

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

function apply() {
  const dark = mode.value === "dark" || (mode.value === "system" && media.matches);
  isDark.value = dark;
  document.documentElement.dataset.theme = dark ? "dark" : "light";
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
