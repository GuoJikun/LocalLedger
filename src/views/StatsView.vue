<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref, watch } from "vue";
import * as echarts from "echarts/core";
import { BarChart, LineChart, PieChart } from "echarts/charts";
import { GridComponent, LegendComponent, TooltipComponent } from "echarts/components";
import { CanvasRenderer } from "echarts/renderers";
import type { EChartsType } from "echarts/core";
import { api } from "@/api/tauri";
import { useTheme } from "@/composables/useTheme";
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
const months = ref(0);
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
  { value: 0, label: "全部月份" },
  { value: 6, label: "最近 6 个月（按数据）" },
  { value: 12, label: "最近 12 个月（按数据）" },
  { value: 24, label: "最近 24 个月（按数据）" },
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
  // v-if 切换会替换 DOM 节点；实例仍绑在旧节点上时必须重建，否则图表画到看不见的旧节点。
  if (existing && !existing.isDisposed() && existing.getDom() === element) {
    return existing;
  }
  if (existing && !existing.isDisposed()) {
    existing.dispose();
  }
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
    barChart?.resize();
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
    pieChart?.resize();
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
    lineChart?.resize();
  }
}

const totalIncome = () => summaries.value.reduce((sum, item) => sum + item.income, 0);
const totalExpense = () => summaries.value.reduce((sum, item) => sum + item.expense, 0);

watch(isDark, () => nextTick(renderCharts));

function resizeChart(chart: EChartsType | null) {
  if (chart && !chart.isDisposed()) chart.resize();
}

function handleResize() {
  resizeChart(barChart);
  resizeChart(pieChart);
  resizeChart(lineChart);
}

onMounted(() => {
  window.addEventListener("resize", handleResize);
  void load();
});
onUnmounted(() => {
  window.removeEventListener("resize", handleResize);
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
      <el-select v-model="months" class="range-select" @change="load">
        <el-option v-for="option in monthOptions" :key="option.value" :label="option.label" :value="option.value" />
      </el-select>
    </header>

    <el-alert v-if="error" type="error" :closable="false" show-icon :title="error" />

    <el-row :gutter="14" class="summary">
      <el-col :xs="24" :sm="8">
        <el-card shadow="never">
          <el-statistic title="区间总收入" :value="totalIncome() / 100" :precision="2" prefix="¥" />
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="8">
        <el-card shadow="never">
          <el-statistic title="区间总支出" :value="totalExpense() / 100" :precision="2" prefix="¥" />
        </el-card>
      </el-col>
      <el-col :xs="24" :sm="8">
        <el-card shadow="never">
          <el-statistic title="净结余" :value="(totalIncome() - totalExpense()) / 100" :precision="2" prefix="¥" />
        </el-card>
      </el-col>
    </el-row>

    <el-card shadow="never" class="page-card">
      <template #header>月度收支</template>
      <div v-if="summaries.length" ref="barEl" class="chart bar"></div>
      <el-empty v-else description="暂无交易数据，导入账单后再来看趋势吧。" />
    </el-card>

    <el-row :gutter="18">
      <el-col :xs="24" :md="12">
        <el-card shadow="never" class="page-card">
          <template #header>
            <div class="card-head">
              <span>分类占比</span>
              <div class="controls">
                <el-select v-model="selectedMonth" class="month-select" @change="loadMonth">
                  <el-option v-for="item in summaries" :key="item.month" :label="item.month" :value="item.month" />
                </el-select>
                <el-select v-model="breakdownKind" class="kind-select" @change="loadMonth">
                  <el-option v-for="option in kindOptions" :key="option.value" :label="option.label" :value="option.value" />
                </el-select>
              </div>
            </div>
          </template>
          <div v-if="slices.length" ref="pieEl" class="chart pie"></div>
          <el-empty v-else :description="`该月暂无${breakdownKind === 'expense' ? '支出' : '收入'}数据。`" />
        </el-card>
      </el-col>

      <el-col :xs="24" :md="12">
        <el-card shadow="never" class="page-card">
          <template #header>每日趋势 · {{ selectedMonth }}</template>
          <div v-if="trend.length" ref="lineEl" class="chart line"></div>
          <el-empty v-else description="该月暂无交易数据。" />
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style scoped>
.stats {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 1280px;
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

.range-select {
  width: 200px;
}

.page-card {
  min-width: 0;
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

.month-select {
  width: 110px;
}

.kind-select {
  width: 90px;
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
</style>
