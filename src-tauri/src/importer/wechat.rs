//! 微信 CSV 解析器：UTF-8 BOM 由通用层处理，这里关注字段映射与转账/红包/退款。

use crate::domain::{NewTransaction, TransactionKind, TransactionSource};
use crate::importer::{
    cell, column_index, detect, optional, parse_amount_cents, parse_kind, parse_time, read_table,
    ImportError, ImportResult, ParsedBill, ParsedRow, Platform, RowIssue,
};

/// 微信账单中的特殊交易类型：转账 / 红包按转账处理，退款按收入处理。
fn kind_from_type(flow: &str, transaction_type: &str) -> Option<TransactionKind> {
    if transaction_type.contains("转账") || transaction_type.contains("红包") {
        return Some(TransactionKind::Transfer);
    }
    if transaction_type.contains("退款") {
        return Some(TransactionKind::Income);
    }
    let parsed = parse_kind(flow);
    if parsed.is_some() {
        return parsed;
    }
    if flow.is_empty() {
        Some(TransactionKind::Transfer)
    } else {
        None
    }
}

pub fn parse(text: &str) -> ImportResult<ParsedBill> {
    if detect(text)? != Platform::Wechat {
        return Err(ImportError::UnknownFormat);
    }

    let (columns, records) = read_table(text, |header| {
        header.iter().any(|value| value.contains("交易时间"))
            && header.iter().any(|value| value.contains("交易单号"))
    })?;

    let time = column_index(&columns, &["交易时间"]);
    let transaction_type = column_index(&columns, &["交易类型"]);
    let counterparty = column_index(&columns, &["交易对方"]);
    let product = column_index(&columns, &["商品"]);
    let flow = column_index(&columns, &["收/支", "收支"]);
    let amount = column_index(&columns, &["金额(元)", "金额"]);
    let payment = column_index(&columns, &["支付方式"]);
    let status = column_index(&columns, &["当前状态", "交易状态"]);
    let id = column_index(&columns, &["交易单号"]);

    let mut rows = Vec::new();
    let mut issues = Vec::new();

    for (line, record) in records {
        let external_id = cell(&record, id);
        if external_id.is_empty() {
            issues.push(RowIssue {
                line,
                reason: "缺少交易单号".to_string(),
            });
            continue;
        }

        let status_text = cell(&record, status);
        if status_text.contains("关闭") || status_text.contains("失败") {
            issues.push(RowIssue {
                line,
                reason: format!("当前状态为「{status_text}」，已跳过"),
            });
            continue;
        }

        let type_text = cell(&record, transaction_type);
        let flow_text = cell(&record, flow);
        let kind = match kind_from_type(&flow_text, &type_text) {
            Some(kind) => kind,
            None => {
                issues.push(RowIssue {
                    line,
                    reason: format!("收/支字段「{flow_text}」不是收支交易，已跳过"),
                });
                continue;
            }
        };

        let amount_cents = match parse_amount_cents(&cell(&record, amount)) {
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

        let product_text = cell(&record, product);
        let note_text = if type_text.is_empty() {
            product_text
        } else {
            format!("[{type_text}] {product_text}")
        };

        rows.push(ParsedRow {
            line,
            input: NewTransaction {
                account_id: "acc_default".to_string(),
                source: TransactionSource::Wechat,
                external_id,
                kind,
                amount: amount_cents,
                counterparty: optional(cell(&record, counterparty)),
                category_id: None,
                note: optional(note_text),
                happened_at,
                payment_method: optional(cell(&record, payment)),
                status: optional(status_text),
                raw_data: None,
            },
            category_hint: None,
        });
    }

    Ok(ParsedBill {
        platform: Platform::Wechat,
        rows,
        issues,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("../../tests/fixtures/wechat_sample.csv");

    #[test]
    fn parses_wechat_sample_with_special_types() {
        let bill = parse(SAMPLE).unwrap();
        assert_eq!(bill.platform, Platform::Wechat);
        assert_eq!(bill.rows.len(), 3);
        assert_eq!(bill.issues.len(), 1);

        assert_eq!(bill.rows[0].input.counterparty.as_deref(), Some("美团外卖"));
        assert_eq!(bill.rows[0].input.amount, 4250);
        assert_eq!(bill.rows[1].input.kind, TransactionKind::Transfer);
        assert_eq!(bill.rows[2].input.kind, TransactionKind::Income);
        assert!(bill.issues[0].reason.contains("已关闭"));
    }
}
