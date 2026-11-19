<script setup lang="ts">
import { useRoute } from "vue-router";

const route = useRoute();
</script>

<template>
  <section class="placeholder">
    <div class="placeholder-card">
      <div class="placeholder-icon">{{ route.meta.icon }}</div>
      <h1>{{ route.meta.title }}</h1>
      <p class="placeholder-desc">{{ route.meta.description }}</p>
      <span v-if="route.meta.milestone" class="badge">
        计划于 {{ route.meta.milestone }} 实现
      </span>
    </div>
  </section>
</template>

<style scoped>
.placeholder {
  display: grid;
  place-items: center;
  min-height: 100%;
}

.placeholder-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 48px 56px;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius);
  box-shadow: var(--shadow);
  text-align: center;
}

.placeholder-icon {
  display: grid;
  place-items: center;
  width: 64px;
  height: 64px;
  border-radius: 18px;
  background: var(--primary-soft);
  color: var(--primary);
  font-size: 30px;
}

h1 {
  margin: 0;
  font-size: 22px;
}

.placeholder-desc {
  margin: 0;
  max-width: 380px;
  color: var(--text-muted);
}

.badge {
  margin-top: 4px;
  padding: 4px 12px;
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--text-muted);
  font-size: 12px;
}
</style>
