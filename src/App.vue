<script setup lang="ts">
import { computed } from "vue";
import { RouterView, useRoute } from "vue-router";

const route = useRoute();

const navItems = [
  { to: "/", icon: "◎", label: "总览" },
  { to: "/transactions", icon: "≡", label: "交易流水" },
  { to: "/import", icon: "↥", label: "导入账单" },
  { to: "/categories", icon: "▦", label: "分类管理" },
  { to: "/statistics", icon: "◔", label: "统计" },
  { to: "/settings", icon: "⚙", label: "设置" },
];

const activePath = computed(() => route.path);
</script>

<template>
  <div class="shell">
    <aside class="sidebar">
      <div class="brand">
        <div class="brand-mark">净</div>
        <div>
          <strong>净账</strong>
          <small>LocalLedger</small>
        </div>
      </div>

      <el-menu
        :default-active="activePath"
        :default-openeds="[]"
        router
        class="side-menu"
        :ellipsis="false"
      >
        <el-menu-item v-for="item in navItems" :key="item.to" :index="item.to">
          <span class="nav-icon">{{ item.icon }}</span>
          <span>{{ item.label }}</span>
        </el-menu-item>
      </el-menu>

      <div class="sidebar-footer">
        <span class="dot"></span>
        离线优先 · 数据在本地
      </div>
    </aside>

    <main class="content">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.shell {
  display: flex;
  height: 100%;
}

.sidebar {
  display: flex;
  flex-direction: column;
  width: 220px;
  flex-shrink: 0;
  padding: 16px 10px;
  background: var(--surface-vibrant);
  -webkit-backdrop-filter: saturate(180%) blur(20px);
  backdrop-filter: saturate(180%) blur(20px);
  border-right: 1px solid var(--border);
}

.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 8px 18px;
  user-select: none;
}

.brand-mark {
  display: grid;
  place-items: center;
  width: 38px;
  height: 38px;
  border-radius: 11px;
  background: var(--primary);
  color: #fff;
  font-size: 18px;
  font-weight: 700;
}

.brand strong {
  display: block;
  font-size: 16px;
  line-height: 1.2;
}

.brand small {
  display: block;
  color: var(--text-faint);
  font-size: 11px;
}

.side-menu {
  border: 0;
  background: transparent;
}

.side-menu :deep(.el-menu-item) {
  height: 36px;
  line-height: 36px;
  border-radius: var(--radius-sm);
  color: var(--text-muted);
}

.side-menu :deep(.el-menu-item.is-active) {
  color: var(--primary);
  background: var(--primary-soft);
  font-weight: 600;
}

.nav-icon {
  width: 20px;
  text-align: center;
  font-size: 15px;
}

.sidebar-footer {
  margin-top: auto;
  padding: 10px 8px 2px;
  color: var(--text-faint);
  font-size: 11px;
  border-top: 1px solid var(--border);
}

.dot {
  display: inline-block;
  width: 7px;
  height: 7px;
  margin-right: 6px;
  border-radius: 50%;
  background: var(--success);
}

.content {
  flex: 1;
  min-width: 0;
  min-height: 0;
  overflow: auto;
  padding: 24px;
}
</style>
