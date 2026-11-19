import { invoke } from "@tauri-apps/api/core";
import type {
  Account,
  Category,
  CategoryRule,
  CategoryRulePatch,
  DataOverview,
  Device,
  ImportSummary,
  NewCategory,
  NewCategoryRule,
  NewTransaction,
  SyncChange,
  Transaction,
  TransactionFilter,
  TransactionPage,
  TransactionPatch,
} from "@/types/domain";

/** 前端调用 Rust 命令的唯一入口；类型与后端 serde 结构一一对应。 */
export const api = {
  dataOverview: () => invoke<DataOverview>("data_overview"),
  deviceInfo: () => invoke<Device>("device_info"),

  listTransactions: () => invoke<Transaction[]>("list_transactions"),
  searchTransactions: (filter: TransactionFilter) =>
    invoke<TransactionPage>("search_transactions", { filter }),
  getTransaction: (id: string) => invoke<Transaction | null>("get_transaction", { id }),
  createTransaction: (input: NewTransaction) =>
    invoke<Transaction>("create_transaction", { input }),
  updateTransaction: (id: string, patch: TransactionPatch) =>
    invoke<Transaction | null>("update_transaction", { id, patch }),
  deleteTransaction: (id: string) => invoke<boolean>("delete_transaction", { id }),

  listAccounts: () => invoke<Account[]>("list_accounts"),
  listCategories: () => invoke<Category[]>("list_categories"),
  createCategory: (input: NewCategory) => invoke<Category>("create_category", { input }),
  updateCategory: (id: string, patch: Partial<NewCategory>) =>
    invoke<Category | null>("update_category", { id, patch }),
  deleteCategory: (id: string) => invoke<boolean>("delete_category", { id }),

  listRules: () => invoke<CategoryRule[]>("list_rules"),
  createRule: (input: NewCategoryRule) => invoke<CategoryRule>("create_rule", { input }),
  updateRule: (id: string, patch: CategoryRulePatch) =>
    invoke<CategoryRule | null>("update_rule", { id, patch }),
  deleteRule: (id: string) => invoke<boolean>("delete_rule", { id }),

  listSyncChanges: (sinceSeq: number, limit = 500) =>
    invoke<SyncChange[]>("list_sync_changes", { sinceSeq, limit }),
  cursorVersion: () => invoke<number>("cursor_version"),

  previewBill: (path: string) => invoke<string>("preview_bill", { path }),
  importBill: (path: string) => invoke<ImportSummary>("import_bill", { path }),
};
