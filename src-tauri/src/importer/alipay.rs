//! 支付宝 CSV 解析器：跳过元信息行、按表头映射列、处理交易状态。

use crate::domain::{NewTransaction, TransactionSource};
use crate::importer::{
    cell, column_index, detect, optional, parse_amount_cents, parse_kind, parse_time, read_table,
    ImportError, ImportResult, ParsedBill, ParsedRow, Platform, RowIssue,
};

pub fn parse(text: &str) -> ImportResult<ParsedBill> {
    if detect(text)? != Platform::Alipay {
        return Err(ImportError::UnknownFormat);
    }

    let (columns, records) = read_table(text, |header| {
        header.iter().any(|value| value.contains("交易号"))
            && header.iter().any(|value| value.contains("交易时间"))
    })?;

    let id = column_index(&columns, &["交易号", "交易订单号"]);
    let time = column_index(&columns, &["交易时间", "交易创建时间"]);
    let category = column_index(&columns, &["交易分类"]);
    let counterparty = column_index(&columns, &["交易对方"]);
    let product = column_index(&columns, &["商品说明", "商品"]);
    let flow = column_index(&columns, &["收/支", "收支"]);
    let amount = column_index(&columns, &["金额", "金额(元)"]);
    let payment = column_index(&columns, &["收/付款方式", "支付方式"]);
    let status = column_index(&columns, &["交易状态", "当前状态"]);

    let mut rows = Vec::new();
    let mut issues = Vec::new();

    for (line, record) in records {
        let external_id = cell(&record, id);
        if external_id.is_empty() {
            issues.push(RowIssue {
                line,
                reason: "缺少交易号".to_string(),
            });
            continue;
        }

        let status_text = cell(&record, status);
        if !status_text.contains("成功") {
            issues.push(RowIssue {
                line,
                reason: format!("交易状态为「{status_text}」，已跳过"),
            });
            continue;
        }

        let flow_text = cell(&record, flow);
        let kind = match parse_kind(&flow_text) {
            Some(kind) => kind,
            None => {
                issues.push(RowIssue {
                    line,
                    reason: format!("收/支字段「{flow_text}」不是收支交易，已跳过"),
                });
                continue;
            }
        };

        let amount_text = cell(&record, amount);
        let amount_cents = match parse_amount_cents(&amount_text) {
            Ok(value) => value.abs(),
            Err(error) => {
                issues.push(RowIssue {
                    line,
                    reason: error.to_string(),
                });
                continue;
            }
        };

        let happened_at = match parse_time(&cell(&record, time)) {
            Some(value) => value,
            None => {
                issues.push(RowIssue {
                    line,
                    reason: "交易时间格式无法识别".to_string(),
                });
                continue;
            }
        };

        let note = optional(cell(&record, product));
        rows.push(ParsedRow {
            line,
            input: NewTransaction {
                account_id: "acc_default".to_string(),
                source: TransactionSource::Alipay,
                external_id,
                kind,
                amount: amount_cents,
                counterparty: optional(cell(&record, counterparty)),
                category_id: None,
                note,
                happened_at,
                payment_method: optional(cell(&record, payment)),
                status: optional(status_text),
                raw_data: None,
            },
            category_hint: optional(cell(&record, category)),
        });
    }

    Ok(ParsedBill {
        platform: Platform::Alipay,
        rows,
        issues,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::TransactionKind;

    const SAMPLE: &str = include_str!("../../tests/fixtures/alipay_sample.csv");

    #[test]
    fn parses_ali_sample_with_all_key_fields() {
        let bill = parse(SAMPLE).unwrap();
        assert_eq!(bill.platform, Platform::Alipay);
        assert_eq!(bill.rows.len(), 3);
        assert_eq!(bill.issues.len(), 1);

        let first = &bill.rows[0];
        assert_eq!(first.input.source, TransactionSource::Alipay);
        assert_eq!(first.input.external_id, "2026010512000000001");
        assert_eq!(first.input.kind, TransactionKind::Expense);
        assert_eq!(first.input.amount, 3550);
        assert_eq!(first.input.counterparty.as_deref(), Some("美团外卖"));
        assert_eq!(first.input.happened_at, "2026-01-05T04:00:00.000Z");
        assert_eq!(first.category_hint.as_deref(), Some("餐饮美食"));

        let salary = &bill.rows[1];
        assert_eq!(salary.input.kind, TransactionKind::Income);
        assert_eq!(salary.input.amount, 1_200_000);

        assert!(bill.issues[0].reason.contains("交易失败"));
    }
}
