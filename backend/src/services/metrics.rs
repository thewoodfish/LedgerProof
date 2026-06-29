use crate::models::{
    metrics::{FinancialMetrics, LendingPolicy},
    transaction::{Category, Transaction},
};
use anyhow::Result;
use chrono::Utc;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use uuid::Uuid;

pub fn compute(merchant_id: Uuid, transactions: &[Transaction]) -> Result<FinancialMetrics> {
    let monthly_revenue = monthly_sum(transactions, |t| {
        t.category == Category::Revenue.to_string() && t.credit > 0
    });

    let monthly_expenses = monthly_sum(transactions, |t| {
        matches!(
            t.category.as_str(),
            "expense" | "tax" | "loan_repayment"
        ) && t.debit > 0
    });

    let monthly_debt_payments = monthly_sum(transactions, |t| {
        t.category == "loan_repayment" && t.debit > 0
    });

    let monthly_balances = month_last_balance(transactions);

    // ── Metric 1: Average monthly revenue ──────────────────────────────────
    let avg_monthly_revenue = average_values(&monthly_revenue);

    // ── Metric 2: Revenue stability (coefficient of variation × 10000 bps) ─
    let revenue_volatility_bps = coefficient_of_variation_bps(&monthly_revenue);

    // ── Metric 3: Positive cash flow months ───────────────────────────────
    let monthly_cash_flow: HashMap<String, i64> = monthly_revenue
        .keys()
        .map(|m| {
            let rev = monthly_revenue.get(m).copied().unwrap_or(0);
            let exp = monthly_expenses.get(m).copied().unwrap_or(0);
            (m.clone(), rev - exp)
        })
        .collect();

    let positive_cash_flow_months = monthly_cash_flow.values().filter(|&&v| v > 0).count() as i32;

    // ── Metric 4 & 5: Average / minimum balance ───────────────────────────
    let avg_monthly_balance = average_values(&monthly_balances);
    let min_balance = monthly_balances.values().copied().min().unwrap_or(0);

    // ── Metric 6: Revenue growth (consecutive months of growth) ───────────
    let revenue_growth_months = consecutive_growth_months(&monthly_revenue);

    // ── Metric 7 & 14: Transaction frequency ──────────────────────────────
    let monthly_tx_counts = monthly_count(transactions);
    let avg_monthly_tx_count = if monthly_tx_counts.is_empty() {
        0
    } else {
        (monthly_tx_counts.values().sum::<i64>() / monthly_tx_counts.len() as i64) as i32
    };

    // ── Metric 8: Customer concentration ─────────────────────────────────
    // Approximate: group revenue transactions by description prefix (counterparty name)
    let customer_concentration_bps = concentration_bps(transactions, "revenue", true);

    // ── Metric 9: Supplier concentration ─────────────────────────────────
    let supplier_concentration_bps = concentration_bps(transactions, "expense", false);

    // ── Metric 10: Expense stability ─────────────────────────────────────
    let expense_variance_bps = coefficient_of_variation_bps(&monthly_expenses);

    // ── Metric 11: Debt ratio ─────────────────────────────────────────────
    let total_debt: i64 = monthly_debt_payments.values().sum();
    let total_rev: i64 = monthly_revenue.values().sum();
    let debt_ratio_bps = if total_rev == 0 {
        0
    } else {
        ((total_debt as f64 / total_rev as f64) * 10_000.0) as i32
    };

    // ── Metric 12: Loan repayment history ────────────────────────────────
    // Simple heuristic: if any month with loan transactions has a gap month (zero payments)
    // after a non-zero month, flag as missed.
    let has_missed_repayments = detect_missed_repayments(transactions);

    // ── Metric 13: Account age ────────────────────────────────────────────
    let account_age_months = compute_account_age(transactions);

    Ok(FinancialMetrics {
        id: Uuid::new_v4(),
        merchant_id,
        computed_at: Utc::now(),
        monthly_revenue: map_to_json(&monthly_revenue),
        avg_monthly_revenue,
        revenue_volatility_bps,
        monthly_cash_flow: map_to_json(&monthly_cash_flow),
        positive_cash_flow_months,
        avg_monthly_balance,
        min_balance,
        revenue_growth_months,
        avg_monthly_tx_count,
        customer_concentration_bps,
        supplier_concentration_bps,
        expense_variance_bps,
        debt_ratio_bps,
        has_missed_repayments,
        account_age_months,
    })
}

pub fn check_policy(
    metrics: &FinancialMetrics,
    policy: &LendingPolicy,
) -> (bool, Vec<String>) {
    let mut failures = Vec::new();

    if let Some(required) = policy.required_monthly_revenue {
        if metrics.avg_monthly_revenue < required {
            failures.push(format!(
                "avg monthly revenue ₦{:.0} < required ₦{:.0}",
                metrics.avg_monthly_revenue as f64 / 100.0,
                required as f64 / 100.0
            ));
        }
    }
    if let Some(required) = policy.required_avg_balance {
        if metrics.avg_monthly_balance < required {
            failures.push(format!(
                "avg balance ₦{:.0} < required ₦{:.0}",
                metrics.avg_monthly_balance as f64 / 100.0,
                required as f64 / 100.0
            ));
        }
    }
    if let Some(required) = policy.required_positive_cash_flow_months {
        if metrics.positive_cash_flow_months < required {
            failures.push(format!(
                "positive cash flow months {} < required {}",
                metrics.positive_cash_flow_months, required
            ));
        }
    }
    if let Some(max) = policy.max_revenue_volatility_bps {
        if metrics.revenue_volatility_bps > max {
            failures.push(format!(
                "revenue volatility {:.1}% > max {:.1}%",
                metrics.revenue_volatility_bps as f64 / 100.0,
                max as f64 / 100.0
            ));
        }
    }
    if let Some(max) = policy.max_customer_concentration_bps {
        if metrics.customer_concentration_bps > max {
            failures.push(format!(
                "customer concentration {:.1}% > max {:.1}%",
                metrics.customer_concentration_bps as f64 / 100.0,
                max as f64 / 100.0
            ));
        }
    }
    if let Some(max) = policy.max_debt_ratio_bps {
        if metrics.debt_ratio_bps > max {
            failures.push(format!(
                "debt ratio {:.1}% > max {:.1}%",
                metrics.debt_ratio_bps as f64 / 100.0,
                max as f64 / 100.0
            ));
        }
    }
    if policy.require_no_missed_repayments == Some(true) && metrics.has_missed_repayments {
        failures.push("missed loan repayments detected".to_string());
    }
    if let Some(required) = policy.required_account_age_months {
        if metrics.account_age_months < required {
            failures.push(format!(
                "account age {} months < required {} months",
                metrics.account_age_months, required
            ));
        }
    }

    (failures.is_empty(), failures)
}

// ── helpers ────────────────────────────────────────────────────────────────

fn month_key(t: &Transaction) -> String {
    t.date.format("%Y-%m").to_string()
}

fn monthly_sum<F>(transactions: &[Transaction], predicate: F) -> HashMap<String, i64>
where
    F: Fn(&Transaction) -> bool,
{
    let mut map: HashMap<String, i64> = HashMap::new();
    for t in transactions.iter().filter(|t| predicate(t)) {
        let key = month_key(t);
        let amount = if t.credit > 0 { t.credit } else { t.debit };
        *map.entry(key).or_default() += amount;
    }
    map
}

fn monthly_count(transactions: &[Transaction]) -> HashMap<String, i64> {
    let mut map: HashMap<String, i64> = HashMap::new();
    for t in transactions {
        *map.entry(month_key(t)).or_default() += 1;
    }
    map
}

fn month_last_balance(transactions: &[Transaction]) -> HashMap<String, i64> {
    let mut map: HashMap<String, i64> = HashMap::new();
    // Transactions should already be ordered by date from the DB
    for t in transactions {
        map.insert(month_key(t), t.balance);
    }
    map
}

fn average_values(map: &HashMap<String, i64>) -> i64 {
    if map.is_empty() {
        return 0;
    }
    map.values().sum::<i64>() / map.len() as i64
}

fn coefficient_of_variation_bps(map: &HashMap<String, i64>) -> i32 {
    if map.len() < 2 {
        return 0;
    }
    let values: Vec<f64> = map.values().map(|&v| v as f64).collect();
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    if mean == 0.0 {
        return 0;
    }
    let variance =
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
    let std_dev = variance.sqrt();
    ((std_dev / mean) * 10_000.0) as i32
}

fn consecutive_growth_months(map: &HashMap<String, i64>) -> i32 {
    let mut months: Vec<(String, i64)> = map.iter().map(|(k, &v)| (k.clone(), v)).collect();
    months.sort_by_key(|(k, _)| k.clone());

    let mut max_streak = 0i32;
    let mut streak = 0i32;
    for i in 1..months.len() {
        if months[i].1 > months[i - 1].1 {
            streak += 1;
            max_streak = max_streak.max(streak);
        } else {
            streak = 0;
        }
    }
    max_streak
}

fn concentration_bps(transactions: &[Transaction], category: &str, use_credit: bool) -> i32 {
    let mut by_party: HashMap<String, i64> = HashMap::new();
    let mut total: i64 = 0;

    for t in transactions.iter().filter(|t| t.category == category) {
        let amount = if use_credit { t.credit } else { t.debit };
        if amount <= 0 {
            continue;
        }
        // Use first 3 words of description as counterparty key
        let key: String = t
            .description
            .split_whitespace()
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase();
        *by_party.entry(key).or_default() += amount;
        total += amount;
    }

    if total == 0 {
        return 0;
    }
    let max = by_party.values().copied().max().unwrap_or(0);
    ((max as f64 / total as f64) * 10_000.0) as i32
}

fn detect_missed_repayments(transactions: &[Transaction]) -> bool {
    let repayment_months: Vec<String> = transactions
        .iter()
        .filter(|t| t.category == "loan_repayment")
        .map(month_key)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    if repayment_months.len() < 2 {
        return false;
    }

    let mut months = repayment_months.clone();
    months.sort();

    // Check if any two consecutive calendar months are both missing when
    // we expect repayments (i.e. there's a gap in the sequence).
    for i in 1..months.len() {
        let prev = &months[i - 1];
        let curr = &months[i];
        if month_diff(prev, curr) > 1 {
            return true;
        }
    }
    false
}

fn month_diff(a: &str, b: &str) -> i32 {
    fn parts(s: &str) -> (i32, i32) {
        let mut p = s.splitn(2, '-');
        let y = p.next().unwrap_or("0").parse().unwrap_or(0);
        let m = p.next().unwrap_or("0").parse().unwrap_or(0);
        (y, m)
    }
    let (ay, am) = parts(a);
    let (by, bm) = parts(b);
    (by - ay) * 12 + (bm - am)
}

fn compute_account_age(transactions: &[Transaction]) -> i32 {
    if transactions.is_empty() {
        return 0;
    }
    let earliest = transactions.iter().map(|t| t.date).min().unwrap();
    let latest = transactions.iter().map(|t| t.date).max().unwrap();
    let diff = latest.signed_duration_since(earliest);
    (diff.num_days() / 30) as i32
}

fn map_to_json(map: &HashMap<String, i64>) -> Value {
    let mut obj = Map::new();
    for (k, v) in map {
        obj.insert(k.clone(), json!(v));
    }
    Value::Object(obj)
}
