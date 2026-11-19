//! 领域模型：与存储实现无关，金额一律用整数「分」，时间统一为 UTC RFC3339。

use std::fmt;

use chrono::SecondsFormat;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

/// 当前 UTC 时间，统一格式（毫秒精度），便于 LWW 比较。
pub fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

macro_rules! string_enum {
    ($name:ident { $($variant:ident => $text:expr),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text),+
                }
            }

            pub fn parse(value: &str) -> AppResult<Self> {
                match value {
                    $($text => Ok(Self::$variant),)+
                    _ => Err(AppError::Invalid(format!("未知的{}: {}", stringify!($name), value))),
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

string_enum!(TransactionKind {
    Income => "income",
    Expense => "expense",
    Transfer => "transfer",
});

string_enum!(TransactionSource {
    Alipay => "alipay",
    Wechat => "wechat",
    Manual => "manual",
});

string_enum!(EntityType {
    Account => "account",
    Transaction => "transaction",
    Category => "category",
    CategoryRule => "category_rule",
});

string_enum!(SyncOp {
    Insert => "insert",
    Update => "update",
    Delete => "delete",
});

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub created_at: String,
    pub updated_at: String,
    pub device_id: String,
    pub version: i64,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
    pub device_id: String,
    pub version: i64,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: String,
    pub account_id: String,
    pub source: TransactionSource,
    pub external_id: String,
    pub kind: TransactionKind,
    /// 金额，单位：分。
    pub amount: i64,
    pub counterparty: Option<String>,
    pub category_id: Option<String>,
    pub note: Option<String>,
    /// UTC RFC3339。
    pub happened_at: String,
    pub payment_method: Option<String>,
    pub status: Option<String>,
    pub raw_data: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub device_id: String,
    pub version: i64,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryRule {
    pub id: String,
    pub keyword: String,
    pub match_field: String,
    pub category_id: String,
    pub priority: i64,
    pub created_at: String,
    pub updated_at: String,
    pub device_id: String,
    pub version: i64,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncChange {
    pub seq: i64,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub op: SyncOp,
    pub payload: Option<String>,
    pub created_at: String,
    pub device_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct NewTransaction {
    pub account_id: String,
    pub source: TransactionSource,
    pub external_id: String,
    pub kind: TransactionKind,
    pub amount: i64,
    pub counterparty: Option<String>,
    pub category_id: Option<String>,
    pub note: Option<String>,
    pub happened_at: String,
    pub payment_method: Option<String>,
    pub status: Option<String>,
    pub raw_data: Option<String>,
}

impl Default for NewTransaction {
    fn default() -> Self {
        Self {
            account_id: "acc_default".to_string(),
            source: TransactionSource::Manual,
            external_id: String::new(),
            kind: TransactionKind::Expense,
            amount: 0,
            counterparty: None,
            category_id: None,
            note: None,
            happened_at: now(),
            payment_method: None,
            status: None,
            raw_data: None,
        }
    }
}

/// `Option<Option<T>>`：字段缺省表示「不修改」，显式 `null` 表示「清空」。
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct TransactionPatch {
    pub account_id: Option<String>,
    pub kind: Option<TransactionKind>,
    pub amount: Option<i64>,
    pub counterparty: Option<Option<String>>,
    pub category_id: Option<Option<String>>,
    pub note: Option<Option<String>>,
    pub happened_at: Option<String>,
    pub payment_method: Option<Option<String>>,
    pub status: Option<Option<String>>,
    pub raw_data: Option<Option<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct NewCategory {
    pub name: String,
    pub parent_id: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct CategoryPatch {
    pub name: Option<String>,
    pub parent_id: Option<Option<String>>,
    pub icon: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub sort_order: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_enums_roundtrip() {
        assert_eq!(
            TransactionKind::parse("income").unwrap(),
            TransactionKind::Income
        );
        assert_eq!(TransactionKind::Expense.as_str(), "expense");
        assert!(TransactionKind::parse("unknown").is_err());
        assert_eq!(TransactionSource::Wechat.as_str(), "wechat");
        assert_eq!(SyncOp::Delete.as_str(), "delete");
    }

    #[test]
    fn now_is_utc_rfc3339() {
        let value = now();
        assert!(value.ends_with('Z'), "时间应为 UTC: {value}");
        assert!(value.contains('T'));
    }
}
