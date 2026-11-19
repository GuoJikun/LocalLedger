import { createRouter, createWebHashHistory, type RouteRecordRaw } from "vue-router";
import OverviewView from "@/views/OverviewView.vue";
import ImportView from "@/views/ImportView.vue";
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
    component: ModulePlaceholder,
    meta: {
      title: "交易流水",
      icon: "≡",
      description: "列表、搜索与筛选、编辑、手工录入，预计在 M3 实现。",
      milestone: "M3",
    },
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
    component: ModulePlaceholder,
    meta: {
      title: "分类管理",
      icon: "▦",
      description: "分类层级与自动分类规则，预计在 M3 实现。",
      milestone: "M3",
    },
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
