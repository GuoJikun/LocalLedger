import { invoke } from "@tauri-apps/api/core";
import type {
  Account,
  Category,
  DataOverview,
  Device,
  NewCategory,
  NewTransaction,
  SyncChange,
  Transaction,
} from "@/types/domain";

/** 前端调用 Rust 命令的唯一入口；类型与后端 serde 结构一一对应。 */
export const api = {
  dataOverview: () => invoke<DataOverview>("data_overview"),
  deviceInfo: () => invoke<Device>("device_info"),

  listTransactions: () => invoke<Transaction[]>("list_transactions"),
  getTransaction: (id: string) => invoke<Transaction | null>("get_transaction", { id }),
  createTransaction: (input: NewTransaction) =>
    invoke<Transaction>("create_transaction", { input }),
  deleteTransaction: (id: string) => invoke<boolean>("delete_transaction", { id }),

  listAccounts: () => invoke<Account[]>("list_accounts"),
  listCategories: () => invoke<Category[]>("list_categories"),
  createCategory: (input: NewCategory) => invoke<Category>("create_category", { input }),
  deleteCategory: (id: string) => invoke<boolean>("delete_category", { id }),

  listSyncChanges: (sinceSeq: number, limit = 500) =>
    invoke<SyncChange[]>("list_sync_changes", { sinceSeq, limit }),
  cursorVersion: () => invoke<number>("cursor_version"),
};
