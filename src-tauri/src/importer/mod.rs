//! 账单导入通用管线：读字节 → 检测编码 → 识别平台 → 跳过头部 → 列映射 → 逐行解析。
//!
//! 解析与字段映射解耦；平台差异集中在 `alipay` / `wechat` 子模块。

pub mod alipay;
pub mod service;
pub mod wechat;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, SecondsFormat};
use serde::Serialize;
use thiserror::Error;

use crate::domain::TransactionKind;

const MAX_FILE_BYTES: usize = 32 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("文件过大（超过 {0} 字节）")]
    TooLarge(usize),

    #[error("无法识别账单格式（需要支付宝或微信 CSV）")]
    UnknownFormat,

    #[error("未找到账单表头")]
    MissingHeader,

    #[error("金额格式无效: {0}")]
    InvalidAmount(String),

    #[error("CSV 解析失败: {0}")]
    Csv(#[from] csv::Error),

    #[error("数据库错误: {0}")]
    Database(#[from] crate::error::AppError),
}

pub type ImportResult<T> = Result<T, ImportError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Alipay,
    Wechat,
}

impl Platform {
    pub fn label(self) -> &'static str {
        match self {
            Platform::Alipay => "支付宝",
            Platform::Wechat => "微信",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedRow {
    pub line: usize,
    pub input: crate::domain::NewTransaction,
    /// 平台原始分类列，供自动分类规则匹配。
    pub category_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RowIssue {
    pub line: usize,
    pub reason: String,
}

#[derive(Debug)]
pub struct ParsedBill {
    pub platform: Platform,
    pub rows: Vec<ParsedRow>,
    pub issues: Vec<RowIssue>,
}

/// UTF-8（可带 BOM）/ UTF-16 / GBK 自动识别，返回不带 BOM 的 UTF-8 文本。
pub fn decode(bytes: &[u8]) -> ImportResult<String> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(ImportError::TooLarge(bytes.len()));
    }

    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Ok(String::from_utf8_lossy(&bytes[3..]).into_owned());
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return Ok(decode_utf16(bytes, true));
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return Ok(decode_utf16(bytes, false));
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return Ok(text.to_string());
    }

    let (decoded, _, _) = encoding_rs::GBK.decode(bytes);
    Ok(decoded.into_owned())
}

fn decode_utf16(bytes: &[u8], little_endian: bool) -> String {
    let units: Vec<u16> = bytes[2..]
        .chunks_exact(2)
        .map(|pair| {
            if little_endian {
                u16::from_le_bytes([pair[0], pair[1]])
            } else {
                u16::from_be_bytes([pair[0], pair[1]])
            }
        })
        .collect();
    String::from_utf16_lossy(&units)
}

/// 依据头部说明行与表头关键字识别平台。
pub fn detect(text: &str) -> ImportResult<Platform> {
    let head: String = text.lines().take(30).collect::<Vec<_>>().join("\n");
    let has_alipay = head.contains("支付宝")
        || head.contains("交易订单号")
        || (head.contains("交易号") && head.contains("交易分类"));
    let has_wechat =
        head.contains("微信") || (head.contains("交易单号") && head.contains("当前状态"));

    match (has_alipay, has_wechat) {
        (true, _) => Ok(Platform::Alipay),
        (_, true) => Ok(Platform::Wechat),
        _ => Err(ImportError::UnknownFormat),
    }
}

/// 完整解析入口：字节 → 平台 → 行记录。
pub fn parse(bytes: &[u8]) -> ImportResult<ParsedBill> {
    let text = decode(bytes)?;
    match detect(&text)? {
        Platform::Alipay => alipay::parse(&text),
        Platform::Wechat => wechat::parse(&text),
    }
}

pub(crate) type CsvTable = (Vec<String>, Vec<(usize, csv::StringRecord)>);

/// 跳过说明行，找到第一满足条件的表头，返回规范化列名与后续数据行（含行号）。
pub(crate) fn read_table<F>(text: &str, header_predicate: F) -> ImportResult<CsvTable>
where
    F: Fn(&[String]) -> bool,
{
    // 支付宝部分导出为制表符分隔（TSV 内容但扩展名仍是 .csv），按样本自动选择分隔符。
    let sample: String = text.lines().take(20).collect::<Vec<_>>().join("\n");
    let tabs = sample.matches('\t').count();
    let commas = sample.matches(',').count();
    let delimiter = if tabs > commas { b'\t' } else { b',' };

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .trim(csv::Trim::All)
        .delimiter(delimiter)
        .from_reader(text.as_bytes());

    let mut columns: Option<Vec<String>> = None;
    let mut rows = Vec::new();

    for result in reader.records() {
        let record = result?;
        let line = record
            .position()
            .map(|position| position.line() as usize)
            .unwrap_or(0);
        if columns.is_none() {
            let cells: Vec<String> = record
                .iter()
                .map(|value| value.trim().trim_start_matches('\u{feff}').to_string())
                .collect();
            if header_predicate(&cells) {
                columns = Some(cells);
            }
            continue;
        }
        rows.push((line, record));
    }

    columns
        .map(|columns| (columns, rows))
        .ok_or(ImportError::MissingHeader)
}

pub(crate) fn column_index(columns: &[String], aliases: &[&str]) -> Option<usize> {
    columns.iter().position(|column| {
        aliases
            .iter()
            .any(|alias| column == alias || column.contains(alias))
    })
}

pub(crate) fn cell(record: &csv::StringRecord, index: Option<usize>) -> String {
    index
        .and_then(|position| record.get(position))
        .unwrap_or("")
        .trim()
        .to_string()
}

pub(crate) fn optional(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

/// 把 `&#165;` / `&yen;` 之类的 HTML 实体还原为字符（GBK 编码时常见）。
fn expand_entities(raw: &str) -> String {
    let mut expanded = String::new();
    let mut chars = raw.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '&' {
            expanded.push(character);
            continue;
        }
        let mut entity = String::new();
        while let Some(&next) = chars.peek() {
            chars.next();
            if next == ';' {
                break;
            }
            entity.push(next);
            if entity.len() > 12 {
                break;
            }
        }
        if let Some(code) = entity.strip_prefix('#') {
            if let Ok(value) = code.parse::<u32>() {
                if let Some(decoded) = char::from_u32(value) {
                    expanded.push(decoded);
                }
                continue;
            }
        }
        if entity == "yen" {
            expanded.push('¥');
        }
    }
    expanded
}

/// 「¥1,234.5 元」→ 123450 分，手工解析避免浮点误差。
pub(crate) fn parse_amount_cents(raw: &str) -> ImportResult<i64> {
    let normalized = expand_entities(raw);
    let cleaned: String = normalized
        .chars()
        .filter(|character| {
            !character.is_whitespace() && !matches!(character, ',' | '¥' | '￥' | '元' | ' ')
        })
        .collect();

    let (sign, value) = if let Some(rest) = cleaned.strip_prefix('-') {
        (-1i64, rest)
    } else if let Some(rest) = cleaned.strip_prefix('+') {
        (1i64, rest)
    } else {
        (1i64, cleaned.as_str())
    };

    if value.is_empty() {
        return Err(ImportError::InvalidAmount(raw.to_string()));
    }

    let (integer, fraction) = match value.split_once('.') {
        Some((integer, fraction)) => (integer, fraction),
        None => (value, ""),
    };
    let integer_value: i64 = if integer.is_empty() {
        0
    } else {
        integer
            .parse()
            .map_err(|_| ImportError::InvalidAmount(raw.to_string()))?
    };
    let mut fraction_digits: String = fraction.chars().take(2).collect();
    while fraction_digits.len() < 2 {
        fraction_digits.push('0');
    }
    let fraction_value: i64 = fraction_digits.parse().unwrap_or(0);

    Ok(sign * (integer_value * 100 + fraction_value))
}

pub(crate) fn parse_kind(raw: &str) -> Option<TransactionKind> {
    match raw.trim() {
        "收入" | "income" => Some(TransactionKind::Income),
        "支出" | "expense" => Some(TransactionKind::Expense),
        "转账" | "transfer" => Some(TransactionKind::Transfer),
        _ => None,
    }
}

/// 平台账单常见时间格式（含中文日期与无分隔紧凑格式）。
const TIME_FORMATS: &[&str] = &[
    "%Y-%m-%d %H:%M:%S",
    "%Y/%m/%d %H:%M:%S",
    "%Y-%m-%d %H:%M",
    "%Y/%m/%d %H:%M",
    "%Y年%m月%d日 %H:%M:%S",
    "%Y年%m月%d日 %H:%M",
    "%Y%m%d %H%M%S",
    "%Y%m%d%H%M%S",
    "%Y-%m-%dT%H:%M:%S",
];

/// 从任意字符串里按数字段提取 年/月/日/时/分/秒，作为最后的兜底解析。
fn extract_time_parts(value: &str) -> Option<[u32; 6]> {
    let mut runs: Vec<String> = Vec::new();
    let mut current = String::new();
    for character in value.chars() {
        if character.is_ascii_digit() {
            current.push(character);
        } else if !current.is_empty() {
            runs.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        runs.push(current);
    }

    if runs.len() >= 3 {
        let mut parts = [0u32; 6];
        parts[0] = runs[0].parse().ok()?;
        parts[1] = runs[1].parse().ok()?;
        parts[2] = runs[2].parse().ok()?;
        parts[3] = runs
            .get(3)
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        parts[4] = runs
            .get(4)
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        parts[5] = runs
            .get(5)
            .and_then(|value| value.parse().ok())
            .unwrap_or(0);
        return Some(parts);
    }

    // 紧凑格式：20260105120000 / 202601051200 / 20260105
    if runs.len() == 1 {
        let digits = &runs[0];
        if digits.len() >= 8 {
            let mut parts = [0u32; 6];
            parts[0] = digits.get(0..4)?.parse().ok()?;
            parts[1] = digits.get(4..6)?.parse().ok()?;
            parts[2] = digits.get(6..8)?.parse().ok()?;
            if digits.len() >= 12 {
                parts[3] = digits.get(8..10)?.parse().ok()?;
                parts[4] = digits.get(10..12)?.parse().ok()?;
            }
            if digits.len() >= 14 {
                parts[5] = digits.get(12..14)?.parse().ok()?;
            }
            return Some(parts);
        }
    }

    None
}

/// 账单时间按中国大陆时区（UTC+8）解析并归一化为 UTC。
pub(crate) fn parse_time(raw: &str) -> Option<String> {
    let value = raw.trim();
    if value.is_empty() {
        return None;
    }
    if let Ok(datetime) = DateTime::parse_from_rfc3339(value) {
        return Some(to_utc_string(datetime));
    }

    let offset = FixedOffset::east_opt(8 * 3600)?;
    for format in TIME_FORMATS {
        if let Ok(naive) = NaiveDateTime::parse_from_str(value, format) {
            return datetime_with_offset(naive, offset).map(to_utc_string);
        }
    }

    let parts = extract_time_parts(value)?;
    if !(1900..=2200).contains(&parts[0]) {
        return None;
    }
    let date = NaiveDate::from_ymd_opt(parts[0] as i32, parts[1], parts[2])?;
    let time = NaiveTime::from_hms_opt(parts[3], parts[4], parts[5])?;
    datetime_with_offset(date.and_time(time), offset).map(to_utc_string)
}

fn datetime_with_offset(
    naive: NaiveDateTime,
    offset: FixedOffset,
) -> Option<DateTime<FixedOffset>> {
    naive.and_local_timezone(offset).single()
}

fn to_utc_string<T: chrono::TimeZone>(datetime: DateTime<T>) -> String {
    datetime
        .with_timezone(&chrono::Utc)
        .to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amount_parsing_is_exact() {
        assert_eq!(parse_amount_cents("¥35.50").unwrap(), 3550);
        assert_eq!(parse_amount_cents("1,234.5 元").unwrap(), 123450);
        assert_eq!(parse_amount_cents("-28.80").unwrap(), -2880);
        assert_eq!(parse_amount_cents("12").unwrap(), 1200);
        assert_eq!(parse_amount_cents("&#165;35.50").unwrap(), 3550);
        assert!(parse_amount_cents("abc").is_err());
    }

    #[test]
    fn decode_handles_bom_and_gbk() {
        assert_eq!(decode(&[0xEF, 0xBB, 0xBF, b'a', b'b']).unwrap(), "ab");
        let (gbk, _, _) = encoding_rs::GBK.encode("支付宝");
        assert_eq!(decode(&gbk).unwrap(), "支付宝");
    }

    #[test]
    fn time_is_normalized_to_utc() {
        let cases = [
            "2026-01-05 12:00:00",
            "2026/1/5 12:00",
            "2026年1月5日 12:00:00",
            "2026年01月05日 12:00",
            "20260105120000",
            "2026-01-05T12:00:00+08:00",
        ];
        for value in cases {
            assert_eq!(
                parse_time(value).unwrap(),
                "2026-01-05T04:00:00.000Z",
                "无法解析: {value}"
            );
        }
        assert_eq!(parse_time("").unwrap_or_default(), "");
    }
}
