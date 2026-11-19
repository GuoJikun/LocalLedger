<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { ElMessageBox } from "element-plus";
import { api } from "@/api/tauri";
import { amountSign, formatAmount, yuanToFen } from "@/types/domain";
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

function kindLabel(kind: TransactionKind): string {
  return kindOptions.find((item) => item.value === kind)?.label ?? kind;
}

function kindTagType(kind: TransactionKind): "success" | "danger" | "info" {
  if (kind === "income") return "success";
  if (kind === "expense") return "danger";
  return "info";
}

async function load() {
  loading.value = true;
  try {
    const request: TransactionFilter = {
      ...filter,
      kind: filter.kind || undefined,
      category_id: filter.category_id || undefined,
      // 账单时间由后端按 UTC+8 归一化，筛选边界也按同一时区解释。
      from: filter.from ? new Date(`${filter.from}T00:00:00+08:00`).toISOString() : undefined,
      to: filter.to ? new Date(`${filter.to}T23:59:59+08:00`).toISOString() : undefined,
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

function onPageChange(current: number) {
  filter.offset = (current - 1) * filter.limit;
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

function openEdit(row: unknown) {
  const transaction = row as Transaction;
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
  const happenedDate = new Date(form.happened_at);
  if (Number.isNaN(happenedDate.getTime())) {
    formError.value = "时间格式无效";
    return;
  }
  const happenedAt = happenedDate.toISOString();
  const fields = {
    kind: form.kind,
    amount,
    counterparty: form.counterparty || null,
    note: form.note || null,
    category_id: form.category_id || null,
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

async function remove(row: unknown) {
  const transaction = row as Transaction;
  try {
    await ElMessageBox.confirm(
      `确定删除「${transaction.counterparty || transaction.note || "该交易"}」吗？`,
      "删除确认",
      { type: "warning" },
    );
    await api.deleteTransaction(transaction.id);
    await load();
  } catch (cause) {
    if (cause !== "cancel" && cause !== "close") error.value = describe(cause);
  }
}

onMounted(async () => {
  try {
    categories.value = await api.listCategories();
  } catch (cause) {
    error.value = describe(cause);
  }
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
      <el-button type="primary" @click="openNew">＋ 手工记一笔</el-button>
    </header>

    <el-form class="filters" :inline="true" @submit.prevent="applyFilter">
      <el-form-item label="关键词">
        <el-input v-model="filter.keyword" placeholder="对方 / 备注 / 单号" clearable @keyup.enter="applyFilter" />
      </el-form-item>
      <el-form-item label="类型">
        <el-select v-model="filter.kind" placeholder="全部" clearable class="filter-select">
          <el-option v-for="item in kindOptions" :key="item.value" :label="item.label" :value="item.value" />
        </el-select>
      </el-form-item>
      <el-form-item label="分类">
        <el-select v-model="filter.category_id" placeholder="全部" clearable class="filter-select">
          <el-option label="未分类" value="__none__" />
          <el-option v-for="item in categories" :key="item.id" :label="item.name" :value="item.id" />
        </el-select>
      </el-form-item>
      <el-form-item label="起始日期">
        <el-date-picker v-model="filter.from" type="date" value-format="YYYY-MM-DD" placeholder="选择日期" class="filter-date" />
      </el-form-item>
      <el-form-item label="结束日期">
        <el-date-picker v-model="filter.to" type="date" value-format="YYYY-MM-DD" placeholder="选择日期" class="filter-date" />
      </el-form-item>
      <el-form-item>
        <el-button type="primary" @click="applyFilter">筛选</el-button>
        <el-button @click="resetFilter">重置</el-button>
      </el-form-item>
    </el-form>

    <el-alert v-if="error" type="error" :closable="false" show-icon :title="error" />

    <el-card shadow="never" class="table-card">
      <el-table :data="page?.items ?? []" v-loading="loading" size="small" empty-text="没有符合条件的交易">
        <el-table-column label="时间" width="150">
          <template #default="{ row }">
            <span class="muted">{{ new Date(row.happened_at).toLocaleString("zh-CN") }}</span>
          </template>
        </el-table-column>
        <el-table-column label="类型" width="80">
          <template #default="{ row }">
            <el-tag :type="kindTagType(row.kind)" size="small" effect="plain">{{ kindLabel(row.kind) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="对方 / 说明" min-width="180">
          <template #default="{ row }">
            <strong>{{ row.counterparty || "—" }}</strong>
            <div v-if="row.note" class="note">{{ row.note }}</div>
          </template>
        </el-table-column>
        <el-table-column label="分类" width="100">
          <template #default="{ row }">{{ categoryName(row.category_id) }}</template>
        </el-table-column>
        <el-table-column label="金额（元）" width="110" align="right">
          <template #default="{ row }">
            <span :class="['amount', row.kind]">
              {{ amountSign(row.kind) }}{{ formatAmount(row.amount) }}
            </span>
          </template>
        </el-table-column>
        <el-table-column label="来源" width="80">
          <template #default="{ row }"><span class="muted">{{ row.source }}</span></template>
        </el-table-column>
        <el-table-column label="操作" width="110" fixed="right">
          <template #default="{ row }">
            <el-button link type="primary" size="small" @click="openEdit(row)">编辑</el-button>
            <el-button link type="danger" size="small" @click="remove(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="pager">
        <el-pagination
          background
          layout="total, prev, pager, next"
          :current-page="Math.floor(filter.offset / filter.limit) + 1"
          :page-size="filter.limit"
          :total="page?.total ?? 0"
          @current-change="onPageChange"
        />
      </div>
    </el-card>

    <el-dialog v-model="showEditor" :title="editingId ? '编辑交易' : '手工记一笔'" width="420px" :close-on-click-modal="false">
      <el-form label-position="top" @submit.prevent="save">
        <el-form-item label="金额（元）">
          <el-input v-model="form.amount" type="number" min="0.01" step="0.01" />
        </el-form-item>
        <el-form-item label="类型">
          <el-select v-model="form.kind" class="full-width">
            <el-option v-for="item in kindOptions" :key="item.value" :label="item.label" :value="item.value" />
          </el-select>
        </el-form-item>
        <el-form-item label="时间">
          <el-date-picker v-model="form.happened_at" type="datetime" value-format="YYYY-MM-DDTHH:mm" class="full-width" />
        </el-form-item>
        <el-form-item label="对方">
          <el-input v-model="form.counterparty" />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="form.note" />
        </el-form-item>
        <el-form-item label="分类">
          <el-select v-model="form.category_id" class="full-width">
            <el-option label="未分类" value="" />
            <el-option v-for="item in categories" :key="item.id" :label="item.name" :value="item.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="状态">
          <el-input v-model="form.status" placeholder="选填" />
        </el-form-item>
      </el-form>
      <el-alert v-if="formError" type="error" :closable="false" :title="formError" />
      <template #footer>
        <el-button @click="showEditor = false">取消</el-button>
        <el-button type="primary" @click="save">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.transactions {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 1280px;
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

.filters {
  padding: 14px;
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

.table-card {
  box-shadow: none;
}

.muted {
  color: var(--text-muted);
}

.note {
  color: var(--text-faint);
  font-size: 11px;
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

.pager {
  display: flex;
  justify-content: flex-end;
  padding-top: 14px;
}

.full-width {
  width: 100%;
}
</style>
