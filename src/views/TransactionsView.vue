<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import {
  ElButton,
  ElDatePicker,
  ElForm,
  ElFormItem,
  ElInput,
  ElOption,
  ElSelect,
} from "element-plus";
import { api } from "@/api/tauri";
import { formatAmount, yuanToFen } from "@/types/domain";
import type {
  Category,
  Transaction,
  TransactionFilter,
  TransactionKind,
  TransactionPage,
} from "@/types/domain";

const filter = reactive<TransactionFilter>({
  keyword: "",
  kind: undefined,
  category_id: undefined,
  from: "",
  to: "",
  offset: 0,
  limit: 20,
});
const page = ref<TransactionPage | null>(null);
const categories = ref<Category[]>([]);
const loading = ref(false);
const error = ref("");

const showEditor = ref(false);
const editingId = ref<string | null>(null);
const form = reactive({
  amount: "",
  kind: "expense" as TransactionKind,
  counterparty: "",
  note: "",
  category_id: "",
  happened_at: "",
  status: "",
});
const formError = ref("");

const kindOptions: { value: TransactionKind; label: string }[] = [
  { value: "expense", label: "支出" },
  { value: "income", label: "收入" },
  { value: "transfer", label: "转账" },
];

function describe(value: unknown): string {
  return value instanceof Error ? value.message : String(value);
}

function toLocalInput(iso: string): string {
  const date = new Date(iso);
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

function categoryName(id: string | null): string {
  if (!id) return "未分类";
  return categories.value.find((item) => item.id === id)?.name ?? id;
}

async function load() {
  loading.value = true;
  try {
    const request: TransactionFilter = {
      ...filter,
      kind: filter.kind || undefined,
      category_id: filter.category_id || undefined,
      from: filter.from ? new Date(filter.from).toISOString() : undefined,
      to: filter.to ? new Date(`${filter.to}T23:59:59`).toISOString() : undefined,
    };
    page.value = await api.searchTransactions(request);
    error.value = "";
  } catch (cause) {
    error.value = describe(cause);
  } finally {
    loading.value = false;
  }
}

function applyFilter() {
  filter.offset = 0;
  void load();
}

function resetFilter() {
  Object.assign(filter, {
    keyword: "",
    kind: undefined,
    category_id: undefined,
    from: "",
    to: "",
    offset: 0,
    limit: 20,
  });
  void load();
}

function movePage(delta: number) {
  filter.offset = Math.max(0, filter.offset + delta * filter.limit);
  void load();
}

function openNew() {
  editingId.value = null;
  Object.assign(form, {
    amount: "",
    kind: "expense",
    counterparty: "",
    note: "",
    category_id: "",
    happened_at: toLocalInput(new Date().toISOString()),
    status: "",
  });
  formError.value = "";
  showEditor.value = true;
}

function openEdit(transaction: Transaction) {
  editingId.value = transaction.id;
  Object.assign(form, {
    amount: formatAmount(transaction.amount),
    kind: transaction.kind,
    counterparty: transaction.counterparty ?? "",
    note: transaction.note ?? "",
    category_id: transaction.category_id ?? "",
    happened_at: toLocalInput(transaction.happened_at),
    status: transaction.status ?? "",
  });
  formError.value = "";
  showEditor.value = true;
}

async function save() {
  const amount = yuanToFen(form.amount);
  if (!Number.isFinite(amount) || amount <= 0) {
    formError.value = "请输入有效金额（大于 0 元）";
    return;
  }
  const happenedAt = new Date(form.happened_at).toISOString();
  if (Number.isNaN(new Date(happenedAt).getTime())) {
    formError.value = "时间格式无效";
    return;
  }
  const categoryId = form.category_id || null;
  const fields = {
    kind: form.kind,
    amount,
    counterparty: form.counterparty || null,
    note: form.note || null,
    category_id: categoryId,
    happened_at: happenedAt,
    status: form.status || null,
  };

  try {
    if (editingId.value) {
      await api.updateTransaction(editingId.value, fields);
    } else {
      await api.createTransaction({
        account_id: "acc_default",
        source: "manual",
        external_id: `manual-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
        ...fields,
        payment_method: null,
        raw_data: null,
      });
    }
    showEditor.value = false;
    await load();
  } catch (cause) {
    formError.value = describe(cause);
  }
}

async function remove(transaction: Transaction) {
  if (!window.confirm(`确定删除「${transaction.counterparty || transaction.note || "该交易"}」吗？`)) {
    return;
  }
  try {
    await api.deleteTransaction(transaction.id);
    await load();
  } catch (cause) {
    error.value = describe(cause);
  }
}

onMounted(async () => {
  categories.value = await api.listCategories();
  await load();
});
</script>

<template>
  <div class="transactions">
    <header class="hero">
      <div>
        <h1>交易流水</h1>
        <p>搜索、筛选、编辑与手工录入；所有改动即时保存到本地数据库。</p>
      </div>
      <button type="button" class="primary" @click="openNew">＋ 手工记一笔</button>
    </header>

    <ElForm class="filters" :inline="true" @submit.prevent="applyFilter">
      <ElFormItem label="关键词">
        <ElInput
          v-model="filter.keyword"
          placeholder="对方 / 备注 / 单号"
          clearable
          @keyup.enter="applyFilter"
        />
      </ElFormItem>
      <ElFormItem label="类型">
        <ElSelect v-model="filter.kind" placeholder="全部" clearable class="filter-select">
          <ElOption
            v-for="item in kindOptions"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </ElSelect>
      </ElFormItem>
      <ElFormItem label="分类">
        <ElSelect v-model="filter.category_id" placeholder="全部" clearable class="filter-select">
          <ElOption label="未分类" value="__none__" />
          <ElOption
            v-for="item in categories"
            :key="item.id"
            :label="item.name"
            :value="item.id"
          />
        </ElSelect>
      </ElFormItem>
      <ElFormItem label="起始日期">
        <ElDatePicker
          v-model="filter.from"
          type="date"
          value-format="YYYY-MM-DD"
          placeholder="选择日期"
          class="filter-date"
        />
      </ElFormItem>
      <ElFormItem label="结束日期">
        <ElDatePicker
          v-model="filter.to"
          type="date"
          value-format="YYYY-MM-DD"
          placeholder="选择日期"
          class="filter-date"
        />
      </ElFormItem>
      <ElFormItem>
        <ElButton type="primary" @click="applyFilter">筛选</ElButton>
        <ElButton @click="resetFilter">重置</ElButton>
      </ElFormItem>
    </ElForm>

    <p v-if="error" class="error">{{ error }}</p>

    <section class="table-card">
      <table>
        <thead>
          <tr>
            <th>时间</th><th>类型</th><th>对方 / 说明</th><th>分类</th>
            <th class="right">金额（元）</th><th>来源</th><th>操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="item in page?.items ?? []" :key="item.id">
            <td class="muted">{{ new Date(item.happened_at).toLocaleString("zh-CN") }}</td>
            <td>
              <span class="tag" :class="item.kind">
                {{ kindOptions.find((option) => option.value === item.kind)?.label }}
              </span>
            </td>
            <td>
              <strong>{{ item.counterparty || "—" }}</strong>
              <small v-if="item.note">{{ item.note }}</small>
            </td>
            <td>{{ categoryName(item.category_id) }}</td>
            <td class="right amount" :class="item.kind">
              {{ item.kind === "income" ? "+" : "-" }}{{ formatAmount(item.amount) }}
            </td>
            <td class="muted">{{ item.source }}</td>
            <td class="actions-cell">
              <button type="button" class="link" @click="openEdit(item)">编辑</button>
              <button type="button" class="link danger" @click="remove(item)">删除</button>
            </td>
          </tr>
          <tr v-if="!page?.items.length">
            <td colspan="7" class="empty">没有符合条件的交易。</td>
          </tr>
        </tbody>
      </table>
    </section>

    <footer class="pager">
      <span>共 {{ page?.total ?? 0 }} 条 · {{ loading ? "加载中…" : "已保存" }}</span>
      <div>
        <button type="button" class="ghost" :disabled="filter.offset === 0" @click="movePage(-1)">上一页</button>
        <button
          type="button"
          class="ghost"
          :disabled="filter.offset + filter.limit >= (page?.total ?? 0)"
          @click="movePage(1)"
        >下一页</button>
      </div>
    </footer>

    <div v-if="showEditor" class="overlay" @click.self="showEditor = false">
      <form class="editor" @submit.prevent="save">
        <h2>{{ editingId ? "编辑交易" : "手工记一笔" }}</h2>
        <label>金额（元）<input v-model="form.amount" type="number" step="0.01" min="0.01" /></label>
        <label>类型
          <select v-model="form.kind">
            <option v-for="item in kindOptions" :key="item.value" :value="item.value">{{ item.label }}</option>
          </select>
        </label>
        <label>时间<input v-model="form.happened_at" type="datetime-local" /></label>
        <label>对方<input v-model="form.counterparty" type="text" /></label>
        <label>备注<input v-model="form.note" type="text" /></label>
        <label>分类
          <select v-model="form.category_id">
            <option value="">未分类</option>
            <option v-for="item in categories" :key="item.id" :value="item.id">{{ item.name }}</option>
          </select>
        </label>
        <label>状态<input v-model="form.status" type="text" placeholder="选填" /></label>
        <p v-if="formError" class="error">{{ formError }}</p>
        <div class="editor-actions">
          <button type="button" class="ghost" @click="showEditor = false">取消</button>
          <button type="submit" class="primary">保存</button>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.transactions {
  display: flex;
  flex-direction: column;
  gap: 18px;
  max-width: 1200px;
  margin: 0 auto;
}

.hero {
  display: flex;
  align-items: center;
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

button:not(.el-button) {
  padding: 7px 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text);
  font: inherit;
}

button.primary {
  border-color: var(--primary);
  background: var(--primary);
  color: #fff;
}

button.ghost:hover:not(:disabled),
button.link:hover {
  background: var(--surface-hover);
}

button:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.filters {
  padding: 16px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
}

.filters :deep(.el-form-item) {
  margin-bottom: 0;
}

.filter-select {
  width: 120px;
}

.filter-date {
  width: 150px;
}

.editor label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: var(--text-muted);
  font-size: 12px;
}

.editor input,
.editor select {
  padding: 7px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg);
  color: var(--text);
  font: inherit;
}

.table-card {
  overflow-x: auto;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

th,
td {
  padding: 10px 14px;
  border-bottom: 1px solid var(--border);
  text-align: left;
}

th {
  color: var(--text-muted);
  font-weight: 500;
  white-space: nowrap;
}

td small {
  display: block;
  color: var(--text-faint);
  font-size: 11px;
}

.right {
  text-align: right;
}

.amount.expense {
  color: var(--danger);
}

.amount.income {
  color: var(--success);
}

.muted {
  color: var(--text-muted);
}

.tag {
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 11px;
  background: var(--surface-2);
}

.tag.income {
  color: var(--success);
}

.tag.expense {
  color: var(--danger);
}

.actions-cell {
  white-space: nowrap;
}

button.link {
  padding: 4px 8px;
  border: 0;
  background: transparent;
  color: var(--primary);
}

button.link.danger {
  color: var(--danger);
}

.empty {
  padding: 40px;
  text-align: center;
  color: var(--text-faint);
}

.pager {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--text-muted);
  font-size: 12px;
}

.pager div {
  display: flex;
  gap: 8px;
}

.error {
  margin: 0;
  color: var(--danger);
  font-size: 12px;
}

.overlay {
  position: fixed;
  inset: 0;
  display: grid;
  place-items: center;
  background: rgba(0, 0, 0, 0.45);
  z-index: 20;
}

.editor {
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 380px;
  padding: 24px;
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: var(--shadow);
}

.editor h2 {
  margin: 0 0 4px;
  font-size: 18px;
}

.editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 8px;
}
</style>
