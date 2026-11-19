<script setup lang="ts">
import { useAppInfo } from "@/composables/useAppInfo";

const { version, dataDir, error, load } = useAppInfo();

const milestones = [
  { code: "M0", title: "脚手架", status: "进行中" },
  { code: "M1", title: "数据底座", status: "待开始" },
  { code: "M2", title: "账单导入", status: "待开始" },
  { code: "M3", title: "流水与分类", status: "待开始" },
  { code: "M4", title: "统计", status: "待开始" },
  { code: "M5", title: "MVP 发布", status: "待开始" },
  { code: "M6", title: "局域网同步", status: "待开始" },
];
</script>

<template>
  <div class="overview">
    <header class="hero">
      <div>
        <h1>欢迎使用净账</h1>
        <p>数据只留在自己电脑：本地运行、离线优先、无账号、无埋点。</p>
      </div>
      <span class="hero-tag">MVP · v{{ version }}</span>
    </header>

    <div class="grid">
      <section class="card">
        <h2>运行环境</h2>
        <dl>
          <div>
            <dt>后端版本</dt>
            <dd>v{{ version }}</dd>
          </div>
          <div>
            <dt>数据目录</dt>
            <dd class="path">{{ dataDir }}</dd>
          </div>
        </dl>
        <p v-if="error" class="warn">浏览器预览模式下无法调用 Rust 命令：{{ error }}</p>
        <button class="refresh" type="button" @click="load">重新读取</button>
      </section>

      <section class="card">
        <h2>里程碑</h2>
        <ul class="milestones">
          <li v-for="item in milestones" :key="item.code">
            <span class="code">{{ item.code }}</span>
            <span class="title">{{ item.title }}</span>
            <span class="status" :class="{ active: item.status === '进行中' }">
              {{ item.status }}
            </span>
          </li>
        </ul>
      </section>
    </div>
  </div>
</template>

<style scoped>
.overview {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 980px;
  margin: 0 auto;
}

.hero {
  display: flex;
  align-items: flex-start;
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

.hero-tag {
  flex-shrink: 0;
  padding: 6px 14px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--surface);
  color: var(--text-muted);
  font-size: 12px;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 20px;
}

.card {
  padding: 22px 24px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
}

.card h2 {
  margin: 0 0 16px;
  font-size: 16px;
}

dl {
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

dl > div {
  display: grid;
  grid-template-columns: 80px 1fr;
  gap: 12px;
}

dt {
  color: var(--text-muted);
}

dd {
  margin: 0;
  word-break: break-all;
}

dd.path {
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  font-size: 12px;
}

.warn {
  margin: 14px 0 0;
  color: var(--warning);
  font-size: 12px;
}

.refresh {
  margin-top: 16px;
  padding: 6px 14px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text);
}

.refresh:hover {
  background: var(--surface-hover);
}

.milestones {
  margin: 0;
  padding: 0;
  list-style: none;
}

.milestones li {
  display: grid;
  grid-template-columns: 48px 1fr auto;
  gap: 10px;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
}

.milestones li:last-child {
  border-bottom: 0;
}

.code {
  color: var(--primary);
  font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
  font-size: 12px;
}

.status {
  color: var(--text-faint);
  font-size: 12px;
}

.status.active {
  color: var(--success);
}
</style>
