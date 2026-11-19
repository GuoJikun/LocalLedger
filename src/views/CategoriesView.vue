<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { api } from "@/api/tauri";
import type { Category, CategoryRule } from "@/types/domain";

const categories = ref<Category[]>([]);
const rules = ref<CategoryRule[]>([]);
const error = ref("");

const showCategoryEditor = ref(false);
const categoryId = ref<string | null>(null);
const categoryForm = reactive({ name: "", parent_id: "", icon: "", color: "", sort_order: 0 });

const showRuleEditor = ref(false);
const ruleId = ref<string | null>(null);
const ruleForm = reactive({ keyword: "", match_field: "counterparty", category_id: "", priority: 0 });

const matchFields = [
  { value: "counterparty", label: "交易对方" },
  { value: "note", label: "备注 / 商品" },
  { value: "category", label: "平台原始分类" },
];

const categoryById = computed(() => new Map(categories.value.map((item) => [item.id, item])));

function describe(value: unknown): string {
  return value instanceof Error ? value.message : String(value);
}

function depth(category: Category): number {
  let level = 0;
  let parent = category.parent_id;
  while (parent && level < 8) {
    level += 1;
    parent = categoryById.value.get(parent)?.parent_id ?? null;
  }
  return level;
}

function categoryName(id: string): string {
  return categoryById.value.get(id)?.name ?? id;
}

async function load() {
  try {
    categories.value = await api.listCategories();
    rules.value = await api.listRules();
    error.value = "";
  } catch (cause) {
    error.value = describe(cause);
  }
}

function openCategory(category?: Category) {
  categoryId.value = category?.id ?? null;
  Object.assign(categoryForm, {
    name: category?.name ?? "",
    parent_id: category?.parent_id ?? "",
    icon: category?.icon ?? "",
    color: category?.color ?? "",
    sort_order: category?.sort_order ?? 0,
  });
  showCategoryEditor.value = true;
}

async function saveCategory() {
  if (!categoryForm.name.trim()) {
    error.value = "分类名称不能为空";
    return;
  }
  const input = {
    name: categoryForm.name.trim(),
    parent_id: categoryForm.parent_id || null,
    icon: categoryForm.icon || null,
    color: categoryForm.color || null,
    sort_order: Number(categoryForm.sort_order),
  };
  try {
    if (categoryId.value) {
      await api.updateCategory(categoryId.value, input);
    } else {
      await api.createCategory(input);
    }
    showCategoryEditor.value = false;
    await load();
  } catch (cause) {
    error.value = describe(cause);
  }
}

async function removeCategory(category: Category) {
  if (!window.confirm(`删除分类「${category.name}」？该操作为软删除。`)) return;
  try {
    await api.deleteCategory(category.id);
    await load();
  } catch (cause) {
    error.value = describe(cause);
  }
}

function openRule(rule?: CategoryRule) {
  ruleId.value = rule?.id ?? null;
  Object.assign(ruleForm, {
    keyword: rule?.keyword ?? "",
    match_field: rule?.match_field ?? "counterparty",
    category_id: rule?.category_id ?? categories.value[0]?.id ?? "",
    priority: rule?.priority ?? 0,
  });
  showRuleEditor.value = true;
}

async function saveRule() {
  if (!ruleForm.keyword.trim() || !ruleForm.category_id) {
    error.value = "请填写关键词并选择分类";
    return;
  }
  const input = {
    keyword: ruleForm.keyword.trim(),
    match_field: ruleForm.match_field,
    category_id: ruleForm.category_id,
    priority: Number(ruleForm.priority),
  };
  try {
    if (ruleId.value) {
      await api.updateRule(ruleId.value, input);
    } else {
      await api.createRule(input);
    }
    showRuleEditor.value = false;
    await load();
  } catch (cause) {
    error.value = describe(cause);
  }
}

async function removeRule(rule: CategoryRule) {
  if (!window.confirm(`删除规则「${rule.keyword}」？`)) return;
  try {
    await api.deleteRule(rule.id);
    await load();
  } catch (cause) {
    error.value = describe(cause);
  }
}

onMounted(load);
</script>

<template>
  <div class="categories">
    <header class="hero">
      <div>
        <h1>分类管理</h1>
        <p>分类支持层级；自动分类规则按优先级匹配，导入账单时自动套用。</p>
      </div>
    </header>

    <p v-if="error" class="error">{{ error }}</p>

    <div class="grid">
      <section class="card">
        <div class="card-head">
          <h2>分类</h2>
          <button type="button" class="primary" @click="openCategory()">＋ 新建</button>
        </div>
        <ul class="tree">
          <li v-for="item in categories" :key="item.id" :style="{ paddingLeft: `${depth(item) * 20}px` }">
            <span class="icon">{{ item.icon || "▫" }}</span>
            <span class="name">{{ item.name }}</span>
            <span class="order">#{{ item.sort_order }}</span>
            <button type="button" class="link" @click="openCategory(item)">编辑</button>
            <button type="button" class="link danger" @click="removeCategory(item)">删除</button>
          </li>
        </ul>
      </section>

      <section class="card">
        <div class="card-head">
          <h2>自动分类规则</h2>
          <button type="button" class="primary" @click="openRule()">＋ 新建规则</button>
        </div>
        <table>
          <thead><tr><th>关键词</th><th>匹配字段</th><th>目标分类</th><th>优先级</th><th></th></tr></thead>
          <tbody>
            <tr v-for="rule in rules" :key="rule.id">
              <td><strong>{{ rule.keyword }}</strong></td>
              <td>{{ matchFields.find((field) => field.value === rule.match_field)?.label ?? rule.match_field }}</td>
              <td>{{ categoryName(rule.category_id) }}</td>
              <td>{{ rule.priority }}</td>
              <td class="row-actions">
                <button type="button" class="link" @click="openRule(rule)">编辑</button>
                <button type="button" class="link danger" @click="removeRule(rule)">删除</button>
              </td>
            </tr>
            <tr v-if="!rules.length"><td colspan="5" class="empty">暂无规则</td></tr>
          </tbody>
        </table>
      </section>
    </div>

    <div v-if="showCategoryEditor" class="overlay" @click.self="showCategoryEditor = false">
      <form class="editor" @submit.prevent="saveCategory">
        <h2>{{ categoryId ? "编辑分类" : "新建分类" }}</h2>
        <label>名称<input v-model="categoryForm.name" type="text" /></label>
        <label>父分类
          <select v-model="categoryForm.parent_id">
            <option value="">无（顶级）</option>
            <option v-for="item in categories" :key="item.id" :value="item.id">{{ item.name }}</option>
          </select>
        </label>
        <label>图标<input v-model="categoryForm.icon" type="text" placeholder=" emoji 或符号" /></label>
        <label>颜色<input v-model="categoryForm.color" type="text" placeholder="#2563eb" /></label>
        <label>排序<input v-model.number="categoryForm.sort_order" type="number" /></label>
        <div class="editor-actions">
          <button type="button" class="ghost" @click="showCategoryEditor = false">取消</button>
          <button type="submit" class="primary">保存</button>
        </div>
      </form>
    </div>

    <div v-if="showRuleEditor" class="overlay" @click.self="showRuleEditor = false">
      <form class="editor" @submit.prevent="saveRule">
        <h2>{{ ruleId ? "编辑规则" : "新建规则" }}</h2>
        <label>关键词<input v-model="ruleForm.keyword" type="text" placeholder="包含此文字即命中" /></label>
        <label>匹配字段
          <select v-model="ruleForm.match_field">
            <option v-for="field in matchFields" :key="field.value" :value="field.value">{{ field.label }}</option>
          </select>
        </label>
        <label>目标分类
          <select v-model="ruleForm.category_id">
            <option v-for="item in categories" :key="item.id" :value="item.id">{{ item.name }}</option>
          </select>
        </label>
        <label>优先级<input v-model.number="ruleForm.priority" type="number" /></label>
        <p class="hint">数值越大越先匹配。</p>
        <div class="editor-actions">
          <button type="button" class="ghost" @click="showRuleEditor = false">取消</button>
          <button type="submit" class="primary">保存</button>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.categories {
  max-width: 1200px;
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

.grid {
  display: grid;
  grid-template-columns: minmax(280px, 1fr) minmax(420px, 1.4fr);
  gap: 18px;
}

.card {
  padding: 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: var(--shadow);
  min-width: 0;
}

.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}

.card h2 {
  margin: 0;
  font-size: 16px;
}

button {
  padding: 6px 14px;
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

button.ghost:hover,
button.link:hover {
  background: var(--surface-hover);
}

button.link {
  padding: 3px 6px;
  border: 0;
  background: transparent;
  color: var(--primary);
}

button.link.danger {
  color: var(--danger);
}

.tree {
  margin: 0;
  padding: 0;
  list-style: none;
}

.tree li {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 0;
  border-bottom: 1px solid var(--border);
}

.icon {
  width: 20px;
  text-align: center;
}

.name {
  flex: 1;
}

.order {
  color: var(--text-faint);
  font-size: 11px;
}

table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}

th,
td {
  padding: 8px 6px;
  border-bottom: 1px solid var(--border);
  text-align: left;
}

th {
  color: var(--text-muted);
  font-weight: 500;
}

.row-actions {
  white-space: nowrap;
}

.empty {
  padding: 24px;
  text-align: center;
  color: var(--text-faint);
}

.error {
  margin: 0;
  color: var(--danger);
  font-size: 12px;
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
  padding: 7px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--bg);
  color: var(--text);
  font: inherit;
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
  margin: 0;
  font-size: 18px;
}

.hint {
  margin: 0;
  color: var(--text-faint);
  font-size: 11px;
}

.editor-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 8px;
}
</style>
