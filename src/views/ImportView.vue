<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { api } from "@/api/tauri";
import type { ImportSummary } from "@/types/domain";

const selectedPath = ref("");
const platform = ref("");
const importing = ref(false);
const dragging = ref(false);
const result = ref<ImportSummary | null>(null);
const message = ref("");

let unlistenDrop: UnlistenFn | null = null;

onMounted(async () => {
  unlistenDrop = await getCurrentWebview().onDragDropEvent((event) => {
    if (event.payload.type === "enter") dragging.value = true;
    else if (event.payload.type === "leave") dragging.value = false;
    else if (event.payload.type === "drop") {
      dragging.value = false;
      const path = event.payload.paths[0];
      if (path) void selectFile(path);
    }
  });
});

onUnmounted(() => unlistenDrop?.());

function describe(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

async function selectFile(path: string) {
  selectedPath.value = path;
  result.value = null;
  message.value = "";
  try {
    platform.value = await api.previewBill(path);
  } catch (error) {
    platform.value = "";
    message.value = describe(error);
  }
}

async function chooseFile() {
  try {
    const path = await open({
      multiple: false,
      filters: [{ name: "账单 CSV", extensions: ["csv"] }],
    });
    if (typeof path === "string") await selectFile(path);
  } catch (error) {
    message.value = describe(error);
  }
}

async function runImport() {
  if (!selectedPath.value || importing.value) return;
  importing.value = true;
  message.value = "";
  try {
    result.value = await api.importBill(selectedPath.value);
  } catch (error) {
    message.value = describe(error);
  } finally {
    importing.value = false;
  }
}
</script>

<template>
  <div class="import-page">
    <header class="hero">
      <h1>导入账单</h1>
      <p>支持支付宝（GBK）与微信（UTF-8 BOM）导出的 CSV；重复导入不会产生重复数据。</p>
    </header>

    <div
      class="drop-zone"
      :class="{ dragging }"
      @dragover.prevent
      @drop.prevent
      @click="chooseFile"
    >
      <div class="drop-icon">↥</div>
      <strong>{{ selectedPath || "拖入账单文件，或点击选择" }}</strong>
      <span v-if="platform">已识别：{{ platform }}账单</span>
      <span v-else>仅在本机读取，不会上传任何文件</span>
    </div>

    <div class="actions">
      <button type="button" class="secondary" @click="chooseFile">选择文件</button>
      <button type="button" :disabled="!selectedPath || importing" @click="runImport">
        {{ importing ? "导入中…" : "开始导入" }}
      </button>
      <span v-if="message" class="message">{{ message }}</span>
    </div>

    <section v-if="result" class="result">
      <h2>导入结果 · {{ result.platform }}</h2>
      <div class="stats">
        <div><strong>{{ result.imported }}</strong><span>成功</span></div>
        <div><strong>{{ result.duplicates }}</strong><span>重复</span></div>
        <div><strong>{{ result.skipped }}</strong><span>跳过</span></div>
        <div><strong>{{ result.classified }}</strong><span>自动分类</span></div>
        <div><strong>{{ result.total_rows }}</strong><span>总行数</span></div>
      </div>
      <div v-if="result.errors.length" class="errors">
        <h3>未导入的行</h3>
        <table>
          <thead><tr><th>行号</th><th>原因</th></tr></thead>
          <tbody>
            <tr v-for="issue in result.errors" :key="issue.line">
              <td>{{ issue.line }}</td>
              <td>{{ issue.reason }}</td>
            </tr>
          </tbody>
        </table>
      </div>
      <p v-else class="all-good">全部行均已处理。</p>
    </section>
  </div>
</template>

<style scoped>
.import-page {
  max-width: 860px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.hero h1 {
  margin: 0 0 6px;
  font-size: 26px;
}

.hero p {
  margin: 0;
  color: var(--text-muted);
}

.drop-zone {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  min-height: 220px;
  padding: 32px;
  border: 2px dashed var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  cursor: pointer;
  transition: border-color 0.15s, background 0.15s;
}

.drop-zone:hover,
.drop-zone.dragging {
  border-color: var(--primary);
  background: var(--primary-soft);
}

.drop-zone strong {
  max-width: 100%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.drop-zone span {
  color: var(--text-muted);
  font-size: 12px;
}

.drop-icon {
  display: grid;
  place-items: center;
  width: 56px;
  height: 56px;
  border-radius: 16px;
  background: var(--primary-soft);
  color: var(--primary);
  font-size: 28px;
}

.actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

button {
  padding: 8px 20px;
  border: 0;
  border-radius: var(--radius-sm);
  background: var(--primary);
  color: #fff;
}

button.secondary {
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--text);
}

button:hover:not(:disabled) {
  filter: brightness(1.08);
}

button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.message {
  color: var(--danger);
  font-size: 12px;
}

.result {
  padding: 22px 24px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: var(--shadow);
}

.result h2 {
  margin: 0 0 16px;
  font-size: 16px;
}

.stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
  gap: 10px;
}

.stats div {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px;
  border-radius: var(--radius-sm);
  background: var(--surface-2);
}

.stats strong {
  font-size: 22px;
  color: var(--primary);
}

.stats span {
  color: var(--text-muted);
  font-size: 12px;
}

.errors h3 {
  margin: 20px 0 8px;
  font-size: 14px;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}

th,
td {
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
  text-align: left;
}

th {
  color: var(--text-muted);
}

.all-good {
  margin: 18px 0 0;
  color: var(--success);
  font-size: 12px;
}
</style>
