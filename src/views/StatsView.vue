<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import * as echarts from "echarts/core";
import { BarChart, LineChart, PieChart } from "echarts/charts";
import { GridComponent, LegendComponent, TooltipComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import type { EChartsType } from "echarts/core";
import { api } from "@/api/tauri";
import { useTheme } from "@/composables/useTheme";
import { formatAmount } from "@/types/domain";
import type { CategorySlice, DailyPoint, MonthSummary } from "@/types/domain";

echarts.use([
  BarChart,
  LineChart,
  PieChart,
  GridComponent,
  LegendComponent,
  TooltipComponent,
  CanvasRenderer,
]);

const { isDark } = useTheme();
const months = ref(12);
const selectedMonth = ref("");
const breakdownKind = ref<"income" | "expense">("expense");
const summaries = ref<MonthSummary[]>([]);
const slices = ref<CategorySlice[]>([]);
const trend = ref<DailyPoint[]>([]);
const error = ref("");

const barEl = ref<HTMLElement | null>(null);
const pieEl = ref<HTMLElement | null>(null);
const lineEl = ref<HTMLElement | null>(null);
let barChart: EChartsType | null = null;
let pieChart: EChartsType | null = null;
let lineChart: EChartsType | null = null;

const monthOptions = [
  { value: 6, label: "近 6 个月" },
  { value: 12, label: "近 12 个月" },
  { value: 24, label: "近 24 个月" },
];

const kindOptions = [
  { value: "expense", label: "支出" },
  { value: "income", label: "收入" },
];

function describe(value: unknown): string {
  return value instanceof Error ? value.message : String(value);
}

function palette() {
  if (isDark.value) {
    return { text: "#9ba6b8", axis: "#2c3546", income: "#34d399", expense: "#f87171" };
  }
  return { text: "#6b7280", axis: "#e2e6ee", income: "#0f9d6e", expense: "#dc2626" };
}

function yuan(fen: number): number {
  return fen / 100;
}

function currentMonth(): string {
  return new Date().toISOString().slice(0, 7);
}

async function loadMonth() {
  try {
    slices.value = await api.categoryBreakdown(selectedMonth.value || null, breakdownKind.value);
    trend.value = await api.dailyTrend(selectedMonth.value || currentMonth());
    error.value = "";
    await nextTick();
    renderCharts();
  } catch (cause) {
    error.value = describe(cause);
  }
}

async function load() {
  try {
    summaries.value = await api.monthlySummary(months.value);
    if (!selectedMonth.value) {
      selectedMonth.value = summaries.value[0]?.month ?? currentMonth();
    }
    await loadMonth();
  } catch (cause) {
    error.value = describe(cause);
  }
}

function ensureChart(
  element: HTMLElement | null,
  existing: EChartsType | null,
): EChartsType | null {
  if (!element) return null;
  if (existing && !existing.isDisposed()) return existing;
  return echarts.init(element);
}

function renderCharts() {
  const colors = palette();

  if (barEl.value && summaries.value.length) {
    barChart = ensureChart(barEl.value, barChart);
    const ordered = [...summaries.value].reverse();
    barChart?.setOption(
      {
        color: [colors.income, colors.expense],
        tooltip: { trigger: "axis", valueFormatter: (value: number) => `¥${value.toFixed(2)}` },
        legend: { textStyle: { color: colors.text }, top: 0 },
        grid: { left: 56, right: 20, bottom: 28, top: 36 },
        xAxis: {
          type: "category",
          data: ordered.map((item) => item.month),
          axisLine: { lineStyle: { color: colors.axis } },
          axisLabel: { color: colors.text },
        },
        yAxis: {
          type: "value",
          axisLabel: { color: colors.text },
          splitLine: { lineStyle: { color: colors.axis, type: "dashed" } },
        },
        series: [
          { name: "收入", type: "bar", data: ordered.map((item) => yuan(item.income)) },
          { name: "支出", type: "bar", data: ordered.map((item) => yuan(item.expense)) },
        ],
      },
      true,
    );
  }

  if (pieEl.value && slices.value.length) {
    pieChart = ensureChart(pieEl.value, pieChart);
    pieChart?.setOption(
      {
        tooltip: {
          trigger: "item",
          valueFormatter: (value: number) => `¥${value.toFixed(2)}`,
        },
        legend: { textStyle: { color: colors.text }, bottom: 0 },
        series: [
          {
            type: "pie",
            radius: ["40%", "68%"],
            label: { color: colors.text, formatter: "{b}\n{d}%" },
            data: slices.value.map((item) => ({
              name: item.name,
              value: yuan(item.amount),
            })),
          },
        ],
      },
      true,
    );
  }

  if (lineEl.value && trend.value.length) {
    lineChart = ensureChart(lineEl.value, lineChart);
    lineChart?.setOption(
      {
        color: [colors.income, colors.expense],
        tooltip: { trigger: "axis", valueFormatter: (value: number) => `¥${value.toFixed(2)}` },
        legend: { textStyle: { color: colors.text }, top: 0 },
        grid: { left: 56, right: 20, bottom: 28, top: 36 },
        xAxis: {
          type: "category",
          data: trend.value.map((item) => item.day.slice(8)),
          axisLine: { lineStyle: { color: colors.axis } },
          axisLabel: { color: colors.text },
        },
        yAxis: {
          type: "value",
          axisLabel: { color: colors.text },
          splitLine: { lineStyle: { color: colors.axis, type: "dashed" } },
        },
        series: [
          { name: "收入", type: "line", smooth: true, data: trend.value.map((item) => yuan(item.income)) },
          { name: "支出", type: "line", smooth: true, data: trend.value.map((item) => yuan(item.expense)) },
        ],
      },
      true,
    );
  }
}

const totalIncome = () => summaries.value.reduce((sum, item) => sum + item.income, 0);
const totalExpense = () => summaries.value.reduce((sum, item) => sum + item.expense, 0);

watch(isDark, () => nextTick(renderCharts));

onMounted(load);
onUnmounted(() => {
  barChart?.dispose();
  pieChart?.dispose();
  lineChart?.dispose();
});
</script>

<template>
  <div class="stats">
    <header class="hero">
      <div>
        <h1>统计</h1>
        <p>月度收支、分类占比与每日趋势；口径与流水一致（仅统计未删除记录）。</p>
      </div>
      <select v-model.number="months" @change="load">
        <option v-for="option in monthOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
    </header>

    <p v-if="error" class="error">{{ error }}</p>

    <div class="summary">
      <div><span>区间总收入</span><strong class="income">¥{{ formatAmount(totalIncome()) }}</strong></div>
      <div><span>区间总支出</span><strong class="expense">¥{{ formatAmount(totalExpense()) }}</strong></div>
      <div>
        <span>净结余</span>
        <strong>¥{{ formatAmount(totalIncome() - totalExpense()) }}</strong>
      </div>
    </div>

    <section class="card">
      <h2>月度收支</h2>
      <div v-if="summaries.length" ref="barEl" class="chart bar"></div>
      <p v-else class="empty">暂无交易数据，导入账单后再来看趋势吧。</p>
    </section>

    <div class="grid">
      <section class="card">
        <div class="card-head">
          <h2>分类占比</h2>
          <div class="controls">
            <select v-model="selectedMonth" @change="loadMonth">
              <option v-for="item in summaries" :key="item.month" :value="item.month">{{ item.month }}</option>
            </select>
            <select v-model="breakdownKind" @change="loadMonth">
              <option v-for="option in kindOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
            </select>
          </div>
        </div>
        <div v-if="slices.length" ref="pieEl" class="chart pie"></div>
        <p v-else class="empty">该月暂无{{ breakdownKind === 'expense' ? '支出' : '收入' }}数据。</p>
      </section>

      <section class="card">
        <h2>每日趋势 · {{ selectedMonth }}</h2>
        <div v-if="trend.length" ref="lineEl" class="chart line"></div>
        <p v-else class="empty">该月暂无交易数据。</p>
      </section>
    </div>
  </div>
</template>

<style scoped>
.stats {
  display: flex;
  flex-direction: column;
  gap: 18px;
  max-width: 1200px;
  margin: 0 auto;
}

.hero {
  display: flex;
  align-items: center;
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

select {
  padding: 6px 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface);
  color: var(--text);
  font: inherit;
}

.summary {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 14px;
}

.summary div {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 16px 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
}

.summary span {
  color: var(--text-muted);
  font-size: 12px;
}

.summary strong {
  font-size: 20px;
}

.income {
  color: var(--success);
}

.expense {
  color: var(--danger);
}

.card {
  padding: 18px 20px;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface);
  box-shadow: var(--shadow);
  min-width: 0;
}

.card h2 {
  margin: 0 0 12px;
  font-size: 15px;
}

.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.controls {
  display: flex;
  gap: 8px;
}

.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 18px;
}

.chart {
  width: 100%;
}

.chart.bar {
  height: 280px;
}

.chart.pie,
.chart.line {
  height: 300px;
}

.empty {
  display: grid;
  place-items: center;
  height: 240px;
  margin: 0;
  color: var(--text-faint);
  font-size: 12px;
}

.error {
  margin: 0;
  color: var(--danger);
  font-size: 12px;
}
</style>
