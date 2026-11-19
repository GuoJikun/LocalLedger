<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { isTauri } from "@tauri-apps/api/core";
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
  if (!isTauri()) return;
  try {
    unlistenDrop = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "enter") dragging.value = true;
      else if (event.payload.type === "leave") dragging.value = false;
      else if (event.payload.type === "drop") {
        dragging.value = false;
        const path = event.payload.paths[0];
        if (path) void selectFile(path);
      }
    });
  } catch {
    // 拖拽监听不可用时仍可通过点击选择文件。
  }
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
      <el-button @click="chooseFile">选择文件</el-button>
      <el-button type="primary" :loading="importing" :disabled="!selectedPath" @click="runImport">
        开始导入
      </el-button>
    </div>

    <el-alert
      v-if="message"
      type="error"
      :closable="false"
      show-icon
      :title="message"
      class="page-alert"
    />

    <el-card v-if="result" shadow="never" class="result-card">
      <template #header>导入结果 · {{ result.platform }}</template>
      <el-row :gutter="8" class="stat-row">
        <el-col :span="4"><el-statistic title="成功" :value="result.imported" /></el-col>
        <el-col :span="4"><el-statistic title="重复" :value="result.duplicates" /></el-col>
        <el-col :span="4"><el-statistic title="跳过" :value="result.skipped" /></el-col>
        <el-col :span="4"><el-statistic title="自动分类" :value="result.classified" /></el-col>
        <el-col :span="8"><el-statistic title="总行数" :value="result.total_rows" /></el-col>
      </el-row>
      <template v-if="result.errors.length">
        <h3 class="section-title">未导入的行</h3>
        <el-table :data="result.errors" size="small" max-height="320">
          <el-table-column prop="line" label="行号" width="80" />
          <el-table-column prop="reason" label="原因" min-width="200" />
        </el-table>
      </template>
      <el-alert
        v-else
        type="success"
        :closable="false"
        show-icon
        title="全部行均已处理"
      />
    </el-card>

    <el-collapse class="supported">
      <el-collapse-item title="当前支持的账单表头（点击展开对照）" name="headers">
        <el-row :gutter="16">
          <el-col :xs="24" :md="12">
            <div class="header-card">
              <div class="header-title">
                <h3>支付宝</h3>
                <span>逗号或 Tab 分隔均可；自动跳过说明行</span>
              </div>
              <pre>交易时间	交易分类	交易对方	对方账号	商品说明	收/支	金额	 收/付款方式 	交易状态	 交易订单号 	商家订单号	备注</pre>
              <p class="header-note">
                也兼容旧版列名：交易号 / 交易订单号 / 商家订单号，商品说明 / 商品，收/支 / 收支，金额 /
                金额(元)，收/付款方式 / 支付方式，交易状态 / 当前状态。收/支为「不计收支」或交易状态非成功的行会跳过。
              </p>
            </div>
          </el-col>
          <el-col :xs="24" :md="12">
            <div class="header-card">
              <div class="header-title">
                <h3>微信</h3>
                <span>UTF-8 BOM；自动跳过说明行</span>
              </div>
              <pre>交易时间,交易类型,交易对方,商品,收/支,金额(元),支付方式,当前状态,交易单号</pre>
              <p class="header-note">
                必需列：交易时间 + 交易单号。转账 / 红包按转账处理，退款按收入处理；状态为「已关闭 /
                失败」的行会跳过。
              </p>
            </div>
          </el-col>
        </el-row>
      </el-collapse-item>
    </el-collapse>
  </div>
</template>

<style scoped>
.import-page {
  max-width: 980px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 18px;
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
  min-height: 200px;
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
  gap: 10px;
}

.page-alert {
  margin-bottom: 0;
}

.result-card {
  box-shadow: none;
}

.stat-row {
  margin-bottom: 18px;
}

.section-title {
  margin: 0 0 10px;
  font-size: 14px;
}

.supported {
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  overflow: hidden;
}

.header-card {
  min-width: 0;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg);
}

.header-title {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 10px;
}

.header-title h3 {
  margin: 0;
  font-size: 14px;
}

.header-title span {
  color: var(--text-faint);
  font-size: 11px;
}

.header-card pre {
  margin: 0;
  padding: 10px;
  overflow-x: auto;
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text);
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  font-size: 11px;
  line-height: 1.7;
  white-space: pre;
}

.header-note {
  margin: 10px 0 0;
  color: var(--text-muted);
  font-size: 11px;
  line-height: 1.7;
}
</style>
