//! 存储抽象：业务逻辑依赖 trait，SQLite 只是其中一种实现。
//!
//! 方法名按聚合根区分，避免同一个实现体上多个 trait 的同名方法产生歧义。

pub mod sqlite;

pub use sqlite::SqliteRepository;

use crate::domain::{
    Account, Category, CategoryPatch, CategoryRule, CategoryRulePatch, NewCategory,
    NewCategoryRule, NewTransaction, SyncChange, Transaction, TransactionFilter, TransactionPage,
    TransactionPatch, TransactionSource,
};
use crate::error::AppResult;

pub trait TransactionRepository {
    fn insert_transaction(&self, input: &NewTransaction) -> AppResult<Transaction>;
    fn get_transaction(&self, id: &str) -> AppResult<Option<Transaction>>;
    fn find_by_external(
        &self,
        source: TransactionSource,
        external_id: &str,
    ) -> AppResult<Option<Transaction>>;
    fn list_transactions(&self, include_deleted: bool) -> AppResult<Vec<Transaction>>;
    fn update_transaction(
        &self,
        id: &str,
        patch: &TransactionPatch,
    ) -> AppResult<Option<Transaction>>;
    fn soft_delete_transaction(&self, id: &str) -> AppResult<bool>;
    fn search_transactions(&self, filter: &TransactionFilter) -> AppResult<TransactionPage>;
}

pub trait CategoryRepository {
    fn insert_category(&self, input: &NewCategory) -> AppResult<Category>;
    fn get_category(&self, id: &str) -> AppResult<Option<Category>>;
    fn list_categories(&self, include_deleted: bool) -> AppResult<Vec<Category>>;
    fn update_category(&self, id: &str, patch: &CategoryPatch) -> AppResult<Option<Category>>;
    fn soft_delete_category(&self, id: &str) -> AppResult<bool>;
}

pub trait AccountRepository {
    fn list_accounts(&self, include_deleted: bool) -> AppResult<Vec<Account>>;
}

pub trait CategoryRuleRepository {
    fn list_rules(&self) -> AppResult<Vec<CategoryRule>>;
    fn insert_rule(&self, input: &NewCategoryRule) -> AppResult<CategoryRule>;
    fn update_rule(&self, id: &str, patch: &CategoryRulePatch) -> AppResult<Option<CategoryRule>>;
    fn soft_delete_rule(&self, id: &str) -> AppResult<bool>;
}

pub trait SyncChangeRepository {
    fn list_since(&self, since_seq: i64, limit: i64) -> AppResult<Vec<SyncChange>>;
    fn count_since(&self, since_seq: i64) -> AppResult<i64>;
}
