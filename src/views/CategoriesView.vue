<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { ElMessageBox } from "element-plus";
import { api } from "@/api/tauri";
import type { Category, CategoryRule } from "@/types/domain";

interface CategoryNode {
  id: string;
  name: string;
  icon: string;
  sort_order: number;
  children: CategoryNode[];
}

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

const treeData = computed<CategoryNode[]>(() => {
  const nodes = new Map<string, CategoryNode>();
  for (const item of categories.value) {
    nodes.set(item.id, {
      id: item.id,
      name: item.name,
      icon: item.icon || "▫",
      sort_order: item.sort_order,
      children: [],
    });
  }
  const roots: CategoryNode[] = [];
  for (const item of categories.value) {
    const node = nodes.get(item.id)!;
    const parent = item.parent_id ? nodes.get(item.parent_id) : undefined;
    if (parent && parent !== node) parent.children.push(node);
    else roots.push(node);
  }
  return roots;
});

function describe(value: unknown): string {
  return value instanceof Error ? value.message : String(value);
}

function categoryName(id: string): string {
  return categoryById.value.get(id)?.name ?? id;
}

function matchFieldLabel(value: string): string {
  return matchFields.find((field) => field.value === value)?.label ?? value;
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
    if (categoryId.value) await api.updateCategory(categoryId.value, input);
    else await api.createCategory(input);
    showCategoryEditor.value = false;
    await load();
  } catch (cause) {
    error.value = describe(cause);
  }
}

async function removeCategory(category: Category) {
  try {
    await ElMessageBox.confirm(`删除分类「${category.name}」？该操作为软删除。`, "删除确认", {
      type: "warning",
    });
    await api.deleteCategory(category.id);
    await load();
  } catch (cause) {
    if (cause !== "cancel" && cause !== "close") error.value = describe(cause);
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
    if (ruleId.value) await api.updateRule(ruleId.value, input);
    else await api.createRule(input);
    showRuleEditor.value = false;
    await load();
  } catch (cause) {
    error.value = describe(cause);
  }
}

async function removeRule(rule: CategoryRule) {
  try {
    await ElMessageBox.confirm(`删除规则「${rule.keyword}」？`, "删除确认", { type: "warning" });
    await api.deleteRule(rule.id);
    await load();
  } catch (cause) {
    if (cause !== "cancel" && cause !== "close") error.value = describe(cause);
  }
}

onMounted(load);
</script>

<template>
  <div class="categories">
    <header class="hero">
      <h1>分类管理</h1>
      <p>分类支持层级；自动分类规则按优先级匹配，导入账单时自动套用。</p>
    </header>

    <el-alert v-if="error" type="error" :closable="false" show-icon :title="error" />

    <el-row :gutter="18">
      <el-col :xs="24" :md="10">
        <el-card shadow="never" class="page-card">
          <template #header>
            <div class="card-head">
              <span>分类</span>
              <el-button type="primary" size="small" @click="openCategory()">＋ 新建</el-button>
            </div>
          </template>
          <el-tree :data="treeData" node-key="id" default-expand-all :expand-on-click-node="false">
            <template #default="{ data }">
              <div class="tree-node">
                <span class="icon">{{ data.icon }}</span>
                <span class="name">{{ data.name }}</span>
                <span class="order">#{{ data.sort_order }}</span>
                <el-button link type="primary" size="small" @click="openCategory(categoryById.get(data.id))">
                  编辑
                </el-button>
                <el-button link type="danger" size="small" @click="removeCategory(categoryById.get(data.id)!)">
                  删除
                </el-button>
              </div>
            </template>
          </el-tree>
        </el-card>
      </el-col>

      <el-col :xs="24" :md="14">
        <el-card shadow="never" class="page-card">
          <template #header>
            <div class="card-head">
              <span>自动分类规则</span>
              <el-button type="primary" size="small" @click="openRule()">＋ 新建规则</el-button>
            </div>
          </template>
          <el-table :data="rules" size="small" empty-text="暂无规则">
            <el-table-column label="关键词" min-width="100">
              <template #default="{ row }"><strong>{{ row.keyword }}</strong></template>
            </el-table-column>
            <el-table-column label="匹配字段" min-width="110">
              <template #default="{ row }">{{ matchFieldLabel(row.match_field) }}</template>
            </el-table-column>
            <el-table-column label="目标分类" min-width="100">
              <template #default="{ row }">{{ categoryName(row.category_id) }}</template>
            </el-table-column>
            <el-table-column prop="priority" label="优先级" width="80" />
            <el-table-column label="操作" width="110" fixed="right">
              <template #default="{ row }">
                <el-button link type="primary" size="small" @click="openRule(row as CategoryRule)">编辑</el-button>
                <el-button link type="danger" size="small" @click="removeRule(row as CategoryRule)">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>
    </el-row>

    <el-dialog v-model="showCategoryEditor" :title="categoryId ? '编辑分类' : '新建分类'" width="420px" :close-on-click-modal="false">
      <el-form label-position="top">
        <el-form-item label="名称">
          <el-input v-model="categoryForm.name" />
        </el-form-item>
        <el-form-item label="父分类">
          <el-select v-model="categoryForm.parent_id" class="full-width">
            <el-option label="无（顶级）" value="" />
            <el-option v-for="item in categories" :key="item.id" :label="item.name" :value="item.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="图标">
          <el-input v-model="categoryForm.icon" placeholder="emoji 或符号" />
        </el-form-item>
        <el-form-item label="颜色">
          <el-input v-model="categoryForm.color" placeholder="#2563eb" />
        </el-form-item>
        <el-form-item label="排序">
          <el-input-number v-model="categoryForm.sort_order" :min="0" class="full-width" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCategoryEditor = false">取消</el-button>
        <el-button type="primary" @click="saveCategory">保存</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="showRuleEditor" :title="ruleId ? '编辑规则' : '新建规则'" width="420px" :close-on-click-modal="false">
      <el-form label-position="top">
        <el-form-item label="关键词">
          <el-input v-model="ruleForm.keyword" placeholder="包含此文字即命中" />
        </el-form-item>
        <el-form-item label="匹配字段">
          <el-select v-model="ruleForm.match_field" class="full-width">
            <el-option v-for="field in matchFields" :key="field.value" :label="field.label" :value="field.value" />
          </el-select>
        </el-form-item>
        <el-form-item label="目标分类">
          <el-select v-model="ruleForm.category_id" class="full-width">
            <el-option v-for="item in categories" :key="item.id" :label="item.name" :value="item.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="优先级">
          <el-input-number v-model="ruleForm.priority" class="full-width" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showRuleEditor = false">取消</el-button>
        <el-button type="primary" @click="saveRule">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.categories {
  max-width: 1280px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
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

.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.tree-node {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.icon {
  width: 20px;
  text-align: center;
}

.name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.order {
  color: var(--text-faint);
  font-size: 11px;
}

.full-width {
  width: 100%;
}
</style>
