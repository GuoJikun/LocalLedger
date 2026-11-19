<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api } from "@/api/tauri";
import { useAppInfo } from "@/composables/useAppInfo";
import { amountSign, formatAmount, yuanToFen } from "@/types/domain";
import type { DataOverview, Transaction, TransactionKind } from "@/types/domain";

const { version } = useAppInfo();

const overview = ref<DataOverview | null>(null);
const recent = ref<Transaction[]>([]);
const loadError = ref("");

const amount = ref<number>();
const kind = ref<TransactionKind>("expense");
const counterparty = ref("");
const note = ref("");
const saving = ref(false);
const formMessage = ref("");
const formError = ref("");

const kindOptions: { value: TransactionKind; label: string }[] = [
  { value: "expense", label: "支出" },
  { value: "income", label: "收入" },
  { value: "transfer", label: "转账" },
];

const total = computed(() => overview.value?.transaction_count ?? 0);

function describe(value: unknown): string {
  return value instanceof Error ? value.message : String(value);
}

async function loadData() {
  try {
    overview.value = await api.dataOverview();
    recent.value = (await api.listTransactions()).slice(0, 5);
    loadError.value = "";
  } catch (cause) {
    loadError.value = describe(cause);
  }
}

async function save() {
  const fen = yuanToFen(String(amount.value ?? ""));
  if (!Number.isFinite(fen) || fen <= 0) {
    formError.value = "请输入有效金额（大于 0 元）";
    return;
  }
  saving.value = true;
  formError.value = "";
  formMessage.value = "";
  try {
    await api.createTransaction({
      account_id: "acc_default",
      source: "manual",
      external_id: `manual-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      kind: kind.value,
      amount: fen,
      counterparty: counterparty.value || null,
      category_id: null,
      note: note.value || null,
      happened_at: new Date().toISOString(),
      payment_method: null,
      status: null,
      raw_data: null,
    });
    amount.value = undefined;
    counterparty.value = "";
    note.value = "";
    formMessage.value = "已保存";
    await loadData();
  } catch (cause) {
    formError.value = describe(cause);
  } finally {
    saving.value = false;
  }
}

onMounted(loadData);
</script>

<template>
  <div class="overview">
    <header class="hero">
      <div>
        <h1>欢迎使用净账</h1>
        <p>数据只留在自己电脑：本地运行、离线优先、无账号、无埋点。</p>
      </div>
      <el-tag type="info" effect="plain" round>MVP · v{{ version }}</el-tag>
    </header>

    <el-alert
      v-if="loadError"
      type="warning"
      :closable="false"
      show-icon
      title="浏览器预览模式下无法调用 Rust 命令"
      class="page-alert"
    />

    <el-row :gutter="20">
      <el-col :xs="24" :md="12">
        <el-card shadow="never" class="page-card">
          <template #header>数据底座</template>
          <el-row :gutter="8" class="stat-row">
            <el-col :span="6"><el-statistic title="交易" :value="total" /></el-col>
            <el-col :span="6"><el-statistic title="分类" :value="overview?.category_count ?? 0" /></el-col>
            <el-col :span="6"><el-statistic title="账本" :value="overview?.account_count ?? 0" /></el-col>
            <el-col :span="6"><el-statistic title="变更" :value="overview?.pending_changes ?? 0" /></el-col>
          </el-row>
          <el-descriptions :column="1" size="small" class="info">
            <el-descriptions-item label="设备">
              {{ overview?.device_name ?? "—" }}（{{ overview?.device_id ?? "—" }}）
            </el-descriptions-item>
            <el-descriptions-item label="数据库">
              <span class="path">{{ overview?.database_path ?? "—" }}</span>
            </el-descriptions-item>
          </el-descriptions>
        </el-card>
      </el-col>

      <el-col :xs="24" :md="12">
        <el-card shadow="never" class="page-card">
          <template #header>快速记一笔</template>
          <el-form label-position="top" @submit.prevent="save">
            <el-form-item label="金额（元）">
              <el-input-number
                v-model="amount"
                :min="0.01"
                :precision="2"
                :step="1"
                placeholder="0.00"
                class="full-width"
              />
            </el-form-item>
            <el-form-item label="类型">
              <el-select v-model="kind" class="full-width">
                <el-option
                  v-for="item in kindOptions"
                  :key="item.value"
                  :label="item.label"
                  :value="item.value"
                />
              </el-select>
            </el-form-item>
            <el-form-item label="对方">
              <el-input v-model="counterparty" placeholder="选填" />
            </el-form-item>
            <el-form-item label="备注">
              <el-input v-model="note" placeholder="选填" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" :loading="saving" native-type="submit">保存</el-button>
              <span v-if="formMessage" class="success-text">{{ formMessage }}</span>
              <span v-if="formError" class="error-text">{{ formError }}</span>
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>

      <el-col :xs="24" :md="12">
        <el-card shadow="never" class="page-card">
          <template #header>最近交易</template>
          <el-table :data="recent" size="small" empty-text="暂无交易，试试「快速记一笔」">
            <el-table-column label="时间" width="150">
              <template #default="{ row }">
                {{ new Date(row.happened_at).toLocaleString("zh-CN") }}
              </template>
            </el-table-column>
            <el-table-column label="对方 / 说明" min-width="140">
              <template #default="{ row }">
                {{ row.counterparty || row.note || "未命名交易" }}
              </template>
            </el-table-column>
            <el-table-column label="金额" width="110" align="right">
              <template #default="{ row }">
                <span :class="['amount', row.kind]">
                  {{ amountSign(row.kind) }}{{ formatAmount(row.amount) }}
                </span>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style scoped>
.overview {
  display: flex;
  flex-direction: column;
  gap: 18px;
  max-width: 1180px;
  margin: 0 auto;
}

.hero {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
}

.hero h1 {
  margin: 0 0 6px;
  font-size: 26px;
}

.hero p {
  margin: 0;
  color: var(--text-muted);
}

.page-card {
  height: 100%;
}

.stat-row {
  margin-bottom: 18px;
}

.info {
  font-size: 12px;
}

.path {
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  font-size: 11px;
  word-break: break-all;
}

.full-width {
  width: 100%;
}

.amount.expense {
  color: var(--danger);
}

.amount.income {
  color: var(--success);
}

.amount.transfer {
  color: var(--text-muted);
}

.success-text {
  margin-left: 12px;
  color: var(--success);
  font-size: 12px;
}

.error-text {
  margin-left: 12px;
  color: var(--danger);
  font-size: 12px;
}

.page-alert {
  margin-bottom: 4px;
}
</style>
