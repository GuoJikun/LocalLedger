import { createRouter, createWebHashHistory, type RouteRecordRaw } from "vue-router";
import OverviewView from "@/views/OverviewView.vue";
import ImportView from "@/views/ImportView.vue";
import TransactionsView from "@/views/TransactionsView.vue";
import CategoriesView from "@/views/CategoriesView.vue";
import ModulePlaceholder from "@/components/ModulePlaceholder.vue";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    name: "overview",
    component: OverviewView,
    meta: { title: "总览", icon: "◎" },
  },
  {
    path: "/transactions",
    name: "transactions",
    component: TransactionsView,
    meta: { title: "交易流水", icon: "≡" },
  },
  {
    path: "/import",
    name: "import",
    component: ImportView,
    meta: { title: "导入账单", icon: "↥" },
  },
  {
    path: "/categories",
    name: "categories",
    component: CategoriesView,
    meta: { title: "分类管理", icon: "▦" },
  },
  {
    path: "/statistics",
    name: "statistics",
    component: ModulePlaceholder,
    meta: {
      title: "统计",
      icon: "◔",
      description: "月度收支、分类占比与趋势图，预计在 M4 实现。",
      milestone: "M4",
    },
  },
  {
    path: "/settings",
    name: "settings",
    component: ModulePlaceholder,
    meta: {
      title: "设置",
      icon: "⚙",
      description: "备份导出、外观与偏好设置，预计在 M5 实现。",
      milestone: "M5",
    },
  },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
