//! 导入落库：自动分类 → 去重 → 写入，并汇总成功/跳过/错误行。

use std::collections::HashSet;

use serde::Serialize;

use crate::importer::{ImportResult, ParsedBill, RowIssue};
use crate::repository::{CategoryRuleRepository, TransactionRepository};

#[derive(Debug, Serialize)]
pub struct ImportSummary {
    pub platform: String,
    pub total_rows: usize,
    pub imported: usize,
    pub duplicates: usize,
    pub skipped: usize,
    pub classified: usize,
    pub errors: Vec<RowIssue>,
}

fn field_value<'a>(
    field: &str,
    input: &'a crate::domain::NewTransaction,
    category_hint: &'a Option<String>,
) -> Option<&'a str> {
    match field {
        "counterparty" => input.counterparty.as_deref(),
        "note" | "product" => input.note.as_deref(),
        "category" => category_hint.as_deref(),
        _ => None,
    }
}

pub fn persist<R>(repository: &R, bill: ParsedBill) -> ImportResult<ImportSummary>
where
    R: TransactionRepository + CategoryRuleRepository,
{
    let rules = repository.list_rules()?;
    let ParsedBill {
        platform,
        rows,
        mut issues,
    } = bill;
    let total_rows = rows.len() + issues.len();
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut imported = 0;
    let mut duplicates = 0;
    let mut classified = 0;

    for row in &rows {
        let key = (
            row.input.source.as_str().to_string(),
            row.input.external_id.clone(),
        );
        if !seen.insert(key) {
            duplicates += 1;
            continue;
        }
        if repository
            .find_by_external(row.input.source, &row.input.external_id)?
            .is_some()
        {
            duplicates += 1;
            continue;
        }

        let mut input = row.input.clone();
        for rule in &rules {
            let value = field_value(&rule.match_field, &input, &row.category_hint);
            let matched = value
                .map(|value| value.to_lowercase().contains(&rule.keyword.to_lowercase()))
                .unwrap_or(false);
            if matched {
                input.category_id = Some(rule.category_id.clone());
                classified += 1;
                break;
            }
        }

        match repository.insert_transaction(&input) {
            Ok(_) => imported += 1,
            Err(error) => issues.push(RowIssue {
                line: row.line,
                reason: error.to_string(),
            }),
        }
    }

    Ok(ImportSummary {
        platform: platform.label().to_string(),
        total_rows,
        imported,
        duplicates,
        skipped: issues.len(),
        classified,
        errors: issues,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{migrate, open_in_memory};
    use crate::domain::TransactionSource;
    use crate::importer;
    use crate::repository::SqliteRepository;

    fn setup() -> SqliteRepository<'static> {
        // 测试仅用内存库；泄漏连接以获得 'static 生命周期，进程结束自动回收。
        let conn = Box::leak(Box::new(open_in_memory().unwrap()));
        migrate(conn).unwrap();
        SqliteRepository::new(conn, "test-device")
    }

    #[test]
    fn import_is_idempotent_and_classifies() {
        let repo = setup();
        let bytes = include_bytes!("../../tests/fixtures/alipay_sample.csv");
        let first = persist(&repo, importer::parse(bytes).unwrap()).unwrap();
        assert_eq!(first.imported, 3);
        assert_eq!(first.duplicates, 0);
        assert_eq!(first.skipped, 1);
        assert_eq!(first.classified, 3);

        let second = persist(&repo, importer::parse(bytes).unwrap()).unwrap();
        assert_eq!(second.imported, 0);
        assert_eq!(second.duplicates, 3);

        let all = repo.list_transactions(false).unwrap();
        assert_eq!(all.len(), 3);
        let salary = all
            .iter()
            .find(|item| item.kind == crate::domain::TransactionKind::Income)
            .unwrap();
        assert_eq!(salary.category_id.as_deref(), Some("cat_salary"));
        let food = all
            .iter()
            .find(|item| item.counterparty.as_deref() == Some("美团外卖"))
            .unwrap();
        assert_eq!(food.category_id.as_deref(), Some("cat_food"));
    }

    #[test]
    fn wechat_import_normalizes_amount_and_time() {
        let repo = setup();
        let bytes = include_bytes!("../../tests/fixtures/wechat_sample.csv");
        let summary = persist(&repo, importer::parse(bytes).unwrap()).unwrap();
        assert_eq!(summary.imported, 3);
        assert_eq!(summary.errors.len(), 1);

        let all = repo.list_transactions(false).unwrap();
        assert_eq!(all.len(), 3);
        assert!(all
            .iter()
            .all(|item| item.source == TransactionSource::Wechat));
        assert!(all.iter().all(|item| item.happened_at.ends_with('Z')));
    }
}
