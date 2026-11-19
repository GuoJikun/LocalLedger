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
        // 至少 3 列，避免把含关键字的说明行误判为表头。
        header.len() >= 3
            && header
                .iter()
                .any(|value| value.contains("交易号") || value.contains("交易订单号"))
            && header.iter().any(|value| value.contains("交易时间"))
    })?;

    let id = column_index(&columns, &["交易订单号", "交易号", "商家订单号"]);
    let time = column_index(&columns, &["交易时间", "交易创建时间"]);
    let category = column_index(&columns, &["交易分类"]);
    let counterparty = column_index(&columns, &["交易对方"]);
    let product = column_index(&columns, &["商品说明", "商品"]);
    let remark = column_index(&columns, &["备注"]);
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
        // 支付宝的「不计收支」（余额理财、退款原路退回等）不属于收入/支出，必须排除。
        if flow_text.contains("不计收支") {
            issues.push(RowIssue {
                line,
                reason: "收/支为「不计收支」，已跳过".to_string(),
            });
            continue;
        }
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

        let time_text = cell(&record, time);
        let happened_at = match parse_time(&time_text) {
            Some(value) => value,
            None => {
                issues.push(RowIssue {
                    line,
                    reason: format!("交易时间格式无法识别: \"{time_text}\""),
                });
                continue;
            }
        };

        let product_text = cell(&record, product);
        let remark_text = cell(&record, remark);
        let note = match (product_text, remark_text) {
            (product, remark) if !product.is_empty() && !remark.is_empty() => {
                Some(format!("{product} / {remark}"))
            }
            (product, _) if !product.is_empty() => Some(product),
            (_, remark) if !remark.is_empty() => Some(remark),
            _ => None,
        };
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

    const TAB_SAMPLE: &str = include_str!("../../tests/fixtures/alipay_tab_sample.csv");

    #[test]
    fn parses_real_tab_separated_header() {
        // 用户实际导出格式：Tab 分隔、无「交易号」列、列名带首尾空格、含商家订单号与备注。
        assert_eq!(super::super::detect(TAB_SAMPLE).unwrap(), Platform::Alipay);

        let bill = parse(TAB_SAMPLE).unwrap();
        assert_eq!(bill.rows.len(), 3);
        assert_eq!(bill.issues.len(), 2);
        assert!(bill
            .issues
            .iter()
            .any(|issue| issue.reason.contains("不计收支")));

        let first = &bill.rows[0];
        assert_eq!(first.input.external_id, "ALI-TAB-001");
        assert_eq!(first.input.amount, 3550);
        assert_eq!(first.input.counterparty.as_deref(), Some("美团外卖"));
        assert_eq!(first.input.note.as_deref(), Some("午餐 / 工作日午餐"));
        assert_eq!(first.input.status.as_deref(), Some("交易成功"));
        assert_eq!(first.input.payment_method.as_deref(), Some("余额宝"));
        assert_eq!(first.input.happened_at, "2026-01-05T04:00:00.000Z");

        let salary = &bill.rows[1];
        assert_eq!(salary.input.kind, TransactionKind::Income);
        assert_eq!(salary.input.amount, 1_200_000);
        assert_eq!(salary.input.external_id, "ALI-TAB-002");
        assert_eq!(salary.category_hint.as_deref(), Some("工资"));
    }

    #[test]
    fn metadata_line_with_keywords_is_not_taken_as_header() {
        let text = format!("说明：本账单包含交易时间与交易订单号信息\n{TAB_SAMPLE}");
        let bill = parse(&text).unwrap();
        assert_eq!(bill.rows.len(), 3);
        assert_eq!(bill.issues.len(), 2);
    }
}
