<script setup lang="ts">
import { onMounted, ref } from "vue";
import { save, open } from "@tauri-apps/plugin-dialog";
import { api } from "@/api/tauri";
import { useAppInfo } from "@/composables/useAppInfo";
import { useTheme, type ThemeMode } from "@/composables/useTheme";
import type { DataOverview } from "@/types/domain";

const { version, dataDir } = useAppInfo();
const { mode, setMode } = useTheme();
const overview = ref<DataOverview | null>(null);
const message = ref("");
const busy = ref(false);

const themeOptions: { value: ThemeMode; label: string }[] = [
  { value: "light", label: "浅色" },
  { value: "dark", label: "深色" },
  { value: "system", label: "跟随系统" },
];

function describe(value: unknown): string {
  return value instanceof Error ? value.message : String(value);
}

async function load() {
  try {
    overview.value = await api.dataOverview();
  } catch (cause) {
    message.value = describe(cause);
  }
}

async function backup() {
  busy.value = true;
  try {
    const path = await save({
      title: "备份数据库",
      defaultPath: "local-ledger-backup.sqlite3",
      filters: [{ name: "SQLite 数据库", extensions: ["sqlite3", "db"] }],
    });
    if (!path) return;
    await api.backupDatabase(path);
    message.value = "备份已保存。";
  } catch (cause) {
    message.value = describe(cause);
  } finally {
    busy.value = false;
  }
}

async function restore() {
  busy.value = true;
  try {
    const path = await open({
      multiple: false,
      filters: [{ name: "SQLite 数据库", extensions: ["sqlite3", "db"] }],
    });
    if (typeof path !== "string") return;
    if (!window.confirm("恢复备份会覆盖当前全部数据，确定继续吗？")) return;
    await api.restoreDatabase(path);
    message.value = "恢复完成，数据已重新载入。";
    await load();
  } catch (cause) {
    message.value = describe(cause);
  } finally {
    busy.value = false;
  }
}

async function exportCsv() {
  busy.value = true;
  try {
    const path = await save({
      title: "导出流水 CSV",
      defaultPath: "local-ledger-transactions.csv",
      filters: [{ name: "CSV 文件", extensions: ["csv"] }],
    });
    if (!path) return;
    const count = await api.exportTransactionsCsv(path);
    message.value = `已导出 ${count} 条交易。`;
  } catch (cause) {
    message.value = describe(cause);
  } finally {
    busy.value = false;
  }
}

onMounted(load);
</script>

<template>
  <div class="settings">
    <header class="hero">
      <h1>设置</h1>
      <p>所有数据都在本机，备份与导出由你完全掌控。</p>
    </header>

    <p v-if="message" class="message">{{ message }}</p>

    <section class="card">
      <h2>数据管理</h2>
      <div class="actions">
        <button type="button" :disabled="busy" @click="backup">备份数据库</button>
        <button type="button" :disabled="busy" @click="restore">从备份恢复</button>
        <button type="button" :disabled="busy" @click="exportCsv">导出流水 CSV</button>
      </div>
      <p class="hint">恢复会覆盖当前数据；建议先做一次备份。导出为 UTF-8 BOM 编码，可直接用 Excel 打开。</p>
      <dl>
        <div><dt>数据库位置</dt><dd class="path">{{ overview?.database_path ?? "—" }}</dd></div>
        <div><dt>当前设备</dt><dd>{{ overview?.device_name ?? "—" }}</dd></div>
        <div><dt>设备 ID</dt><dd class="path">{{ overview?.device_id ?? "—" }}</dd></div>
      </dl>
    </section>

    <section class="card">
      <h2>外观</h2>
      <div class="theme-options">
        <button
          v-for="option in themeOptions"
          :key="option.value"
          type="button"
          :class="{ selected: mode === option.value }"
          @click="setMode(option.value)"
        >
          {{ option.label }}
        </button>
      </div>
    </section>

    <section class="card">
      <h2>关于</h2>
      <dl>
        <div><dt>应用</dt><dd>净账 LocalLedger</dd></div>
        <div><dt>版本</dt><dd>v{{ version }}</dd></div>
        <div><dt>数据目录</dt><dd class="path">{{ dataDir }}</dd></div>
      </dl>
    </section>
  </div>
</template>

<style scoped>
.settings {
  display: flex;
  flex-direction: column;
  gap: 18px;
  max-width: 820px;
  margin: 0 auto;
}

.hero h1 {
  margin: 0 0 6px;
  font-size: 26px;
}

.hero p {
  margin: 0;
  color: var(--text-muted);
}

.card {
  padding: 20px 24px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: var(--shadow);
}

.card h2 {
  margin: 0 0 16px;
  font-size: 16px;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

button {
  padding: 8px 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text);
  font: inherit;
}

button:hover:not(:disabled) {
  background: var(--surface-hover);
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.theme-options {
  display: flex;
  gap: 8px;
}

button.selected {
  border-color: var(--primary);
  color: var(--primary);
}

.message {
  margin: 0;
  color: var(--success);
  font-size: 13px;
}

.hint {
  margin: 12px 0 0;
  color: var(--text-faint);
  font-size: 12px;
}

dl {
  margin: 18px 0 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

dl > div {
  display: grid;
  grid-template-columns: 100px 1fr;
  gap: 12px;
}

dt {
  color: var(--text-muted);
}

dd {
  margin: 0;
  word-break: break-all;
}

dd.path {
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  font-size: 11px;
}
</style>
