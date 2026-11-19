<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api } from "@/api/tauri";
import { useAppInfo } from "@/composables/useAppInfo";
import { formatAmount, yuanToFen } from "@/types/domain";
import type { DataOverview, Transaction, TransactionKind } from "@/types/domain";

const { version } = useAppInfo();

const overview = ref<DataOverview | null>(null);
const recent = ref<Transaction[]>([]);
const loadError = ref("");

const amount = ref("");
const kind = ref<TransactionKind>("expense");
const counterparty = ref("");
const note = ref("");
const saving = ref(false);
const formMessage = ref("");

const kindOptions: { value: TransactionKind; label: string }[] = [
  { value: "expense", label: "支出" },
  { value: "income", label: "收入" },
  { value: "transfer", label: "转账" },
];

const milestones = [
  { code: "M0", title: "脚手架", status: "已完成" },
  { code: "M1", title: "数据底座", status: "已完成" },
  { code: "M2", title: "账单导入", status: "已完成" },
  { code: "M3", title: "流水与分类", status: "已完成" },
  { code: "M4", title: "统计", status: "已完成" },
  { code: "M5", title: "MVP 发布", status: "已完成" },
  { code: "M6", title: "局域网同步", status: "待开始" },
];

const total = computed(() => overview.value?.transaction_count ?? 0);

async function loadData() {
  try {
    overview.value = await api.dataOverview();
    recent.value = (await api.listTransactions()).slice(0, 5);
    loadError.value = "";
  } catch (e) {
    loadError.value = e instanceof Error ? e.message : String(e);
  }
}

async function save() {
  const fen = yuanToFen(amount.value);
  if (!Number.isFinite(fen) || fen <= 0) {
    formMessage.value = "请输入有效的金额（大于 0 元）";
    return;
  }
  saving.value = true;
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
    amount.value = "";
    counterparty.value = "";
    note.value = "";
    formMessage.value = "已保存";
    await loadData();
  } catch (e) {
    formMessage.value = e instanceof Error ? e.message : String(e);
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
      <span class="hero-tag">MVP · v{{ version }}</span>
    </header>

    <div class="grid">
      <section class="card">
        <h2>数据底座</h2>
        <div class="stats">
          <div><strong>{{ total }}</strong><span>交易</span></div>
          <div><strong>{{ overview?.category_count ?? "—" }}</strong><span>分类</span></div>
          <div><strong>{{ overview?.account_count ?? "—" }}</strong><span>账本</span></div>
          <div><strong>{{ overview?.pending_changes ?? "—" }}</strong><span>变更</span></div>
        </div>
        <dl>
          <div>
            <dt>设备</dt>
            <dd>{{ overview?.device_name ?? "—" }}（{{ overview?.device_id ?? "—" }}）</dd>
          </div>
          <div>
            <dt>数据库</dt>
            <dd class="path">{{ overview?.database_path ?? "—" }}</dd>
          </div>
        </dl>
        <p v-if="loadError" class="warn">浏览器预览模式下无法调用 Rust 命令：{{ loadError }}</p>
      </section>

      <section class="card">
        <h2>快速记一笔</h2>
        <form @submit.prevent="save">
          <label>金额（元）<input v-model="amount" type="number" step="0.01" min="0.01" placeholder="0.00" /></label>
          <label>类型
            <select v-model="kind">
              <option v-for="option in kindOptions" :key="option.value" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </label>
          <label>对方<input v-model="counterparty" type="text" placeholder="选填" /></label>
          <label>备注<input v-model="note" type="text" placeholder="选填" /></label>
          <button type="submit" :disabled="saving">{{ saving ? "保存中…" : "保存" }}</button>
          <span v-if="formMessage" class="form-message">{{ formMessage }}</span>
        </form>
      </section>

      <section class="card">
        <h2>最近交易</h2>
        <ul v-if="recent.length" class="recent">
          <li v-for="item in recent" :key="item.id">
            <span class="party">{{ item.counterparty || item.note || "未命名交易" }}</span>
            <span class="amount" :class="item.kind">
              {{ item.kind === "income" ? "+" : "-" }}{{ formatAmount(item.amount) }}
            </span>
          </li>
        </ul>
        <p v-else class="empty">暂无交易，试试右侧「快速记一笔」。</p>
      </section>

      <section class="card">
        <h2>里程碑</h2>
        <ul class="milestones">
          <li v-for="item in milestones" :key="item.code">
            <span class="code">{{ item.code }}</span>
            <span class="title">{{ item.title }}</span>
            <span class="status" :class="{ active: item.status === '进行中' }">
              {{ item.status }}
            </span>
          </li>
        </ul>
      </section>
    </div>
  </div>
</template>

<style scoped>
.overview {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 1080px;
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

.hero-tag {
  flex-shrink: 0;
  padding: 6px 14px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--surface);
  color: var(--text-muted);
  font-size: 12px;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 20px;
}

.card {
  padding: 22px 24px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
}

.card h2 {
  margin: 0 0 16px;
  font-size: 16px;
}

.stats {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 110px), 1fr));
  gap: 8px;
  margin-bottom: 18px;
}

.stats div {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 10px 4px;
  border-radius: var(--radius-sm);
  background: var(--surface-2);
}

.stats strong {
  font-size: 20px;
  color: var(--primary);
}

.stats span {
  color: var(--text-muted);
  font-size: 11px;
}

dl {
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

dl > div {
  display: grid;
  grid-template-columns: 60px 1fr;
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

.warn {
  margin: 14px 0 0;
  color: var(--warning);
  font-size: 12px;
}

form {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: var(--text-muted);
  font-size: 12px;
}

input,
select {
  padding: 8px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg);
  color: var(--text);
  font: inherit;
}

button {
  align-self: flex-start;
  padding: 7px 20px;
  border: 0;
  border-radius: var(--radius-sm);
  background: var(--primary);
  color: #fff;
}

button:hover:not(:disabled) {
  background: var(--primary-hover);
}

button:disabled {
  opacity: 0.6;
  cursor: wait;
}

.form-message {
  color: var(--success);
  font-size: 12px;
}

.recent {
  margin: 0;
  padding: 0;
  list-style: none;
}

.recent li {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
}

.recent li:last-child {
  border-bottom: 0;
}

.party {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.amount {
  flex-shrink: 0;
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
}

.amount.expense {
  color: var(--danger);
}

.amount.income {
  color: var(--success);
}

.empty {
  margin: 0;
  color: var(--text-faint);
  font-size: 12px;
}

.milestones {
  margin: 0;
  padding: 0;
  list-style: none;
}

.milestones li {
  display: grid;
  grid-template-columns: 48px 1fr auto;
  gap: 10px;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
}

.milestones li:last-child {
  border-bottom: 0;
}

.code {
  color: var(--primary);
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  font-size: 12px;
}

.status {
  color: var(--text-faint);
  font-size: 12px;
}

.status.active {
  color: var(--success);
}
</style>
