export type TransactionKind = "income" | "expense" | "transfer";
export type TransactionSource = "alipay" | "wechat" | "manual";

export interface Device {
  id: string;
  name: string;
  last_seen_at: string;
}

export interface Account {
  id: string;
  name: string;
  kind: string;
  created_at: string;
  updated_at: string;
  device_id: string;
  version: number;
  deleted: boolean;
}

export interface Category {
  id: string;
  name: string;
  parent_id: string | null;
  icon: string | null;
  color: string | null;
  sort_order: number;
  created_at: string;
  updated_at: string;
  device_id: string;
  version: number;
  deleted: boolean;
}

export interface Transaction {
  id: string;
  account_id: string;
  source: TransactionSource;
  external_id: string;
  kind: TransactionKind;
  /** 单位：分 */
  amount: number;
  counterparty: string | null;
  category_id: string | null;
  note: string | null;
  happened_at: string;
  payment_method: string | null;
  status: string | null;
  raw_data: string | null;
  created_at: string;
  updated_at: string;
  device_id: string;
  version: number;
  deleted: boolean;
}

export interface NewTransaction {
  account_id: string;
  source: TransactionSource;
  external_id: string;
  kind: TransactionKind;
  amount: number;
  counterparty: string | null;
  category_id: string | null;
  note: string | null;
  happened_at: string;
  payment_method: string | null;
  status: string | null;
  raw_data: string | null;
}

export interface NewCategory {
  name: string;
  parent_id: string | null;
  icon: string | null;
  color: string | null;
  sort_order: number;
}

export interface DataOverview {
  device_id: string;
  device_name: string;
  database_path: string;
  account_count: number;
  transaction_count: number;
  category_count: number;
  pending_changes: number;
}

export interface TransactionFilter {
  keyword?: string;
  kind?: TransactionKind;
  category_id?: string;
  from?: string;
  to?: string;
  min_amount?: number;
  max_amount?: number;
  offset: number;
  limit: number;
}

export interface TransactionPage {
  items: Transaction[];
  total: number;
}

export interface TransactionPatch {
  account_id?: string;
  kind?: TransactionKind;
  amount?: number;
  counterparty?: string | null;
  category_id?: string | null;
  note?: string | null;
  happened_at?: string;
  payment_method?: string | null;
  status?: string | null;
  raw_data?: string | null;
}

export interface CategoryRule {
  id: string;
  keyword: string;
  match_field: string;
  category_id: string;
  priority: number;
  created_at: string;
  updated_at: string;
  device_id: string;
  version: number;
  deleted: boolean;
}

export interface NewCategoryRule {
  keyword: string;
  match_field: string;
  category_id: string;
  priority: number;
}

export interface CategoryRulePatch {
  keyword?: string;
  match_field?: string;
  category_id?: string;
  priority?: number;
}

export interface MonthSummary {
  month: string;
  income: number;
  expense: number;
  transfer: number;
  count: number;
}

export interface CategorySlice {
  category_id: string;
  name: string;
  amount: number;
  count: number;
  percent: number;
}

export interface DailyPoint {
  day: string;
  income: number;
  expense: number;
}

export interface RowIssue {
  line: number;
  reason: string;
}

export interface ImportSummary {
  platform: string;
  total_rows: number;
  imported: number;
  duplicates: number;
  skipped: number;
  classified: number;
  errors: RowIssue[];
}

export interface SyncChange {
  seq: number;
  entity_type: "account" | "transaction" | "category" | "category_rule";
  entity_id: string;
  op: "insert" | "update" | "delete";
  payload: string | null;
  created_at: string;
  device_id: string;
}

/** 元 → 分，避免浮点误差。 */
export function yuanToFen(yuan: string): number {
  const value = Number(yuan);
  if (!Number.isFinite(value)) return NaN;
  return Math.round(value * 100);
}

/** 金额前缀：收入 +、支出 -、转账无符号。 */
export function amountSign(kind: TransactionKind): string {
  if (kind === "income") return "+";
  if (kind === "expense") return "-";
  return "";
}

/** 分 → 展示用字符串。 */
export function formatAmount(fen: number): string {
  const sign = fen < 0 ? "-" : "";
  const abs = Math.abs(fen);
  return `${sign}${Math.floor(abs / 100)}.${String(abs % 100).padStart(2, "0")}`;
}
