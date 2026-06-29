use crate::models::transaction::RawTransaction;
use anyhow::{anyhow, Result};
use calamine::{open_workbook_from_rs, Data, Reader, Xlsx};
use std::io::Cursor;

/// Parse an XLS/XLSX bank statement into raw transactions.
/// Expects columns (case-insensitive, order detected automatically):
///   Date | Description / Narration | Debit | Credit | Balance
pub fn parse(bytes: &[u8]) -> Result<Vec<RawTransaction>> {
    let cursor = Cursor::new(bytes);
    let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)
        .map_err(|e| anyhow!("Failed to open workbook: {e}"))?;

    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| anyhow!("Workbook has no sheets"))?;

    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| anyhow!("Failed to read sheet '{sheet_name}': {e}"))?;

    let mut rows = range.rows();

    // Find header row — first row containing at least "date" and an amount column
    let headers = loop {
        match rows.next() {
            None => return Err(anyhow!("No header row found in spreadsheet")),
            Some(row) => {
                let lower: Vec<String> = row
                    .iter()
                    .map(|c| cell_to_string(c).to_lowercase())
                    .collect();
                let has_date = lower.iter().any(|h| h.contains("date"));
                let has_amount = lower.iter().any(|h| {
                    h.contains("credit") || h.contains("debit") || h.contains("amount")
                });
                if has_date && has_amount {
                    break lower;
                }
            }
        }
    };

    // Map column names to indices
    let col = |names: &[&str]| -> Option<usize> {
        names
            .iter()
            .find_map(|name| headers.iter().position(|h| h.contains(name)))
    };

    let date_col    = col(&["date"]).ok_or_else(|| anyhow!("No 'date' column found"))?;
    let desc_col    = col(&["narration", "description", "details", "particular", "remark", "memo"]);
    let debit_col   = col(&["debit", "withdrawal", "dr"]);
    let credit_col  = col(&["credit", "deposit", "cr"]);
    let amount_col  = col(&["amount"]); // single-column fallback
    let balance_col = col(&["balance"]);

    if debit_col.is_none() && credit_col.is_none() && amount_col.is_none() {
        return Err(anyhow!(
            "No amount column found (expected 'debit', 'credit', or 'amount')"
        ));
    }

    let mut transactions = Vec::new();

    for row in rows {
        let date_cell = row.get(date_col).unwrap_or(&Data::Empty);
        let date_str = date_from_cell(date_cell);
        let date_str = match date_str {
            Some(d) => d,
            None => continue, // skip blank rows, subtotals, etc.
        };

        let description = desc_col
            .and_then(|i| row.get(i))
            .map(cell_to_string)
            .unwrap_or_default();

        let (debit, credit) = match (debit_col, credit_col) {
            (Some(d), Some(c)) => (cell_to_f64(row, d), cell_to_f64(row, c)),
            _ => {
                if let Some(a) = amount_col {
                    let v = cell_to_f64(row, a);
                    if v >= 0.0 {
                        (0.0, v)
                    } else {
                        (v.abs(), 0.0)
                    }
                } else {
                    (0.0, 0.0)
                }
            }
        };

        let balance = balance_col
            .map(|i| cell_to_f64(row, i))
            .unwrap_or(0.0);

        transactions.push(RawTransaction {
            date: date_str,
            description,
            debit,
            credit,
            balance,
        });
    }

    if transactions.is_empty() {
        return Err(anyhow!(
            "No transactions found. Check that column headers include Date, Description, Debit, Credit, Balance."
        ));
    }

    Ok(transactions)
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.trim().to_string(),
        Data::Float(f) => f.to_string(),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => {
            if let Some(d) = dt.as_datetime().map(|dt| dt.date()) {
                format!("{}", d.format("%Y-%m-%d"))
            } else {
                String::new()
            }
        }
        Data::DateTimeIso(s) => s.clone(),
        _ => String::new(),
    }
}

fn date_from_cell(cell: &Data) -> Option<String> {
    match cell {
        Data::DateTime(dt) => dt.as_datetime().map(|dt| dt.date()).map(|d| format!("{}", d.format("%Y-%m-%d"))),
        Data::DateTimeIso(s) => {
            // ISO strings are already YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS
            Some(s.get(..10).unwrap_or(s).to_string())
        }
        Data::String(s) => parse_date_str(s.trim()),
        _ => None,
    }
}

fn cell_to_f64(row: &[Data], idx: usize) -> f64 {
    row.get(idx)
        .and_then(|c| match c {
            Data::Float(f) => Some(*f),
            Data::Int(i) => Some(*i as f64),
            Data::String(s) => {
                let clean: String = s
                    .chars()
                    .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                    .collect();
                clean.parse().ok()
            }
            _ => None,
        })
        .unwrap_or(0.0)
}

fn parse_date_str(s: &str) -> Option<String> {
    // Already YYYY-MM-DD
    if s.len() >= 10 && s.chars().nth(4) == Some('-') {
        return Some(s[..10].to_string());
    }
    // DD/MM/YYYY or DD-MM-YYYY
    let sep = if s.contains('/') { '/' } else { '-' };
    let parts: Vec<&str> = s.splitn(3, sep).collect();
    if parts.len() == 3 {
        if parts[2].len() == 4 {
            // DD/MM/YYYY
            return Some(format!("{}-{:0>2}-{:0>2}", parts[2], parts[1], parts[0]));
        }
        if parts[0].len() == 4 {
            // YYYY/MM/DD
            return Some(format!("{}-{:0>2}-{:0>2}", parts[0], parts[1], parts[2]));
        }
    }
    None
}
