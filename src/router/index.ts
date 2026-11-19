import { createRouter, createWebHashHistory, type RouteRecordRaw } from "vue-router";
import OverviewView from "@/views/OverviewView.vue";
import ImportView from "@/views/ImportView.vue";
import TransactionsView from "@/views/TransactionsView.vue";
import CategoriesView from "@/views/CategoriesView.vue";
import StatsView from "@/views/StatsView.vue";
import SettingsView from "@/views/SettingsView.vue";

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
    component: StatsView,
    meta: { title: "统计", icon: "◔" },
  },
  {
    path: "/settings",
    name: "settings",
    component: SettingsView,
    meta: { title: "设置", icon: "⚙" },
  },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
