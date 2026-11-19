<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { RouterLink, RouterView } from "vue-router";
import { ElButton, ElIcon } from "element-plus";
import {
  Close,
  CopyDocument,
  FullScreen,
  Minus,
} from "@element-plus/icons-vue";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

const inTauri = isTauri();
const maximized = ref(false);

const navItems = [
  { to: "/", icon: "◎", label: "总览" },
  { to: "/transactions", icon: "≡", label: "交易流水" },
  { to: "/import", icon: "↥", label: "导入账单" },
  { to: "/categories", icon: "▦", label: "分类管理" },
  { to: "/statistics", icon: "◔", label: "统计" },
  { to: "/settings", icon: "⚙", label: "设置" },
];

async function syncMaximized() {
  if (!inTauri) return;
  try {
    maximized.value = await getCurrentWindow().isMaximized();
  } catch {
    // 浏览器预览模式忽略
  }
}

async function minimize() {
  try {
    await getCurrentWindow().minimize();
  } catch {
    // 浏览器预览模式忽略
  }
}

async function toggleMaximize() {
  try {
    await getCurrentWindow().toggleMaximize();
    await syncMaximized();
  } catch {
    // 浏览器预览模式忽略
  }
}

async function closeWindow() {
  try {
    await getCurrentWindow().close();
  } catch {
    // 浏览器预览模式忽略
  }
}

onMounted(() => {
  void syncMaximized();
  if (inTauri) window.addEventListener("resize", syncMaximized);
});

onUnmounted(() => window.removeEventListener("resize", syncMaximized));
</script>

<template>
  <div class="shell">
    <aside class="sidebar">
      <div class="brand" data-tauri-drag-region>
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
        <div class="drag-area" data-tauri-drag-region @dblclick="toggleMaximize"></div>

        <div v-if="inTauri" class="window-controls">
          <ElButton class="win-btn" text circle title="最小化" @click="minimize">
            <ElIcon><Minus /></ElIcon>
          </ElButton>
          <ElButton
            class="win-btn"
            text
            circle
            :title="maximized ? '还原' : '最大化'"
            @click="toggleMaximize"
          >
            <ElIcon><component :is="maximized ? CopyDocument : FullScreen" /></ElIcon>
          </ElButton>
          <ElButton class="win-btn close" text circle title="关闭" @click="closeWindow">
            <ElIcon><Close /></ElIcon>
          </ElButton>
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
  height: 40px;
  flex-shrink: 0;
  padding-left: 16px;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  user-select: none;
}

.drag-area {
  display: flex;
  align-items: center;
  gap: 10px;
  flex: 1;
  height: 100%;
  min-width: 0;
}

.window-controls {
  display: flex;
  align-items: center;
  height: 100%;
  flex-shrink: 0;
}

.win-btn {
  width: 38px;
  height: 40px;
  margin: 0;
  border-radius: 0;
  color: var(--text-muted);
  font-size: 15px;
}

.win-btn:hover {
  background: var(--surface-hover);
  color: var(--text);
}

.win-btn.close:hover {
  background: var(--danger);
  color: #fff;
}

.content {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 24px;
}
</style>
