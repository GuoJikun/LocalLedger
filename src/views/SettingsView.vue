<script setup lang="ts">
import { onMounted, ref } from "vue";
import { ElMessageBox } from "element-plus";
import { save, open } from "@tauri-apps/plugin-dialog";
import { api } from "@/api/tauri";
import { useAppInfo } from "@/composables/useAppInfo";
import { useTheme, type ThemeMode } from "@/composables/useTheme";
import type { DataOverview } from "@/types/domain";

const { version, dataDir } = useAppInfo();
const { mode, setMode } = useTheme();
const overview = ref<DataOverview | null>(null);
const message = ref("");
const messageType = ref<"success" | "error">("success");
const busy = ref(false);

const themeOptions: { value: ThemeMode; label: string }[] = [
  { value: "light", label: "浅色" },
  { value: "dark", label: "深色" },
  { value: "system", label: "跟随系统" },
];

function describe(value: unknown): string {
  return value instanceof Error ? value.message : String(value);
}

function notify(text: string, type: "success" | "error" = "success") {
  message.value = text;
  messageType.value = type;
}

async function load() {
  try {
    overview.value = await api.dataOverview();
  } catch (cause) {
    notify(describe(cause), "error");
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
    notify("备份已保存。");
  } catch (cause) {
    notify(describe(cause), "error");
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
    try {
      await ElMessageBox.confirm("恢复备份会覆盖当前全部数据，确定继续吗？", "恢复确认", {
        type: "warning",
      });
    } catch {
      return;
    }
    await api.restoreDatabase(path);
    notify("恢复完成，数据已重新载入。");
    await load();
  } catch (cause) {
    notify(describe(cause), "error");
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
    notify(`已导出 ${count} 条交易。`);
  } catch (cause) {
    notify(describe(cause), "error");
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

    <el-alert
      v-if="message"
      :type="messageType"
      :closable="false"
      show-icon
      :title="message"
    />

    <el-card shadow="never" class="page-card">
      <template #header>数据管理</template>
      <div class="actions">
        <el-button :loading="busy" @click="backup">备份数据库</el-button>
        <el-button :loading="busy" @click="restore">从备份恢复</el-button>
        <el-button :loading="busy" @click="exportCsv">导出流水 CSV</el-button>
      </div>
      <p class="hint">恢复会覆盖当前数据；建议先做一次备份。导出为 UTF-8 BOM 编码，可直接用 Excel 打开。</p>
      <el-descriptions :column="1" size="small" class="info">
        <el-descriptions-item label="数据库位置">
          <span class="path">{{ overview?.database_path ?? "—" }}</span>
        </el-descriptions-item>
        <el-descriptions-item label="当前设备">{{ overview?.device_name ?? "—" }}</el-descriptions-item>
        <el-descriptions-item label="设备 ID">
          <span class="path">{{ overview?.device_id ?? "—" }}</span>
        </el-descriptions-item>
      </el-descriptions>
    </el-card>

    <el-card shadow="never" class="page-card">
      <template #header>外观</template>
      <el-radio-group :model-value="mode" @update:model-value="setMode($event as ThemeMode)">
        <el-radio-button v-for="option in themeOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </el-radio-button>
      </el-radio-group>
    </el-card>

    <el-card shadow="never" class="page-card">
      <template #header>关于</template>
      <el-descriptions :column="1" size="small">
        <el-descriptions-item label="应用">净账 LocalLedger</el-descriptions-item>
        <el-descriptions-item label="版本">v{{ version }}</el-descriptions-item>
        <el-descriptions-item label="数据目录">
          <span class="path">{{ dataDir }}</span>
        </el-descriptions-item>
      </el-descriptions>
    </el-card>
  </div>
</template>

<style scoped>
.settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 860px;
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

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.hint {
  margin: 12px 0 0;
  color: var(--text-faint);
  font-size: 12px;
}

.info {
  margin-top: 18px;
}

.path {
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  font-size: 11px;
  word-break: break-all;
}
</style>
