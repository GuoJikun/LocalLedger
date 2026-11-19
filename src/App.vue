<script setup lang="ts">
import { computed } from "vue";
import { RouterLink, RouterView, useRoute } from "vue-router";
import { useTheme, type ThemeMode } from "@/composables/useTheme";

const route = useRoute();
const { mode, setMode } = useTheme();

const navItems = computed(() => [
  { to: "/", icon: "◎", label: "总览" },
  { to: "/transactions", icon: "≡", label: "交易流水" },
  { to: "/import", icon: "↥", label: "导入账单" },
  { to: "/categories", icon: "▦", label: "分类管理" },
  { to: "/statistics", icon: "◔", label: "统计" },
  { to: "/settings", icon: "⚙", label: "设置" },
]);

const themeOptions: { value: ThemeMode; label: string }[] = [
  { value: "light", label: "浅色" },
  { value: "dark", label: "深色" },
  { value: "system", label: "跟随系统" },
];

const currentTitle = computed(() => String(route.meta.title ?? "净账"));
</script>

<template>
  <div class="shell">
    <aside class="sidebar">
      <div class="brand">
        <span class="brand-mark">净</span>
        <div>
          <strong>净账</strong>
          <small>LocalLedger</small>
        </div>
      </div>

      <nav>
        <RouterLink
          v-for="item in navItems"
          :key="item.to"
          :to="item.to"
          class="nav-item"
          active-class="active"
          exact-active-class="active"
        >
          <span class="nav-icon">{{ item.icon }}</span>
          <span>{{ item.label }}</span>
        </RouterLink>
      </nav>

      <div class="sidebar-footer">
        <span class="dot"></span>
        离线优先 · 数据在本地
      </div>
    </aside>

    <div class="main">
      <header class="topbar">
        <h1 class="page-title">{{ currentTitle }}</h1>
        <div class="theme-switch" role="group" aria-label="主题">
          <button
            v-for="option in themeOptions"
            :key="option.value"
            type="button"
            :class="{ selected: mode === option.value }"
            @click="setMode(option.value)"
          >
            {{ option.label }}
          </button>
        </div>
      </header>

      <main class="content">
        <RouterView />
      </main>
    </div>
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
  padding: 20px 14px;
  background: var(--surface);
  border-right: 1px solid var(--border);
}

.brand {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 8px 24px;
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

nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  color: var(--text-muted);
  transition: background 0.15s, color 0.15s;
}

.nav-item:hover {
  background: var(--surface-hover);
  color: var(--text);
}

.nav-item.active {
  background: var(--primary-soft);
  color: var(--primary);
  font-weight: 600;
}

.nav-icon {
  width: 20px;
  text-align: center;
  font-size: 16px;
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

.main {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
}

.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 60px;
  flex-shrink: 0;
  padding: 0 28px;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
}

.page-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.theme-switch {
  display: flex;
  gap: 2px;
  padding: 3px;
  border-radius: 999px;
  background: var(--surface-2);
}

.theme-switch button {
  padding: 5px 12px;
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: var(--text-muted);
  font-size: 12px;
}

.theme-switch button.selected {
  background: var(--surface);
  color: var(--text);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

.content {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 28px;
}
</style>
