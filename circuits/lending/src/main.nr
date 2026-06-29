// LedgerProof Underwriting Circuit -- lending_v1
//
// Private inputs: actual financial metrics (never revealed to lender)
// Public inputs:  thresholds configured by the lender
//
// A valid proof means: "the private values satisfy the public thresholds"
// without disclosing the private values themselves.

fn main(
    // PRIVATE INPUTS (merchant's actual financial data)
    // Monthly revenue in kobo (100 kobo = 1 NGN), 6-month window
    monthly_revenue: [u64; 6],
    // Monthly expenses in kobo, 6-month window
    monthly_expenses: [u64; 6],
    // Monthly average balances in kobo, 6-month window
    monthly_balances: [u64; 6],
    // Revenue coefficient of variation x 10,000 (pre-computed off-chain)
    // e.g. 15% volatility = 1500
    revenue_volatility_bps: u64,
    // Largest customer as % of revenue x 10,000 (pre-computed off-chain)
    customer_concentration_bps: u64,
    // Debt payments as % of revenue x 10,000 (pre-computed off-chain)
    debt_ratio_bps: u64,
    // 1 if any repayment was missed, 0 otherwise
    has_missed_repayments: u64,
    // Number of months the account has been active
    account_age_months: u64,

    // PUBLIC INPUTS (lender's underwriting thresholds)
    // Minimum average monthly revenue in kobo
    required_monthly_revenue: pub u64,
    // Minimum average balance in kobo
    required_avg_balance: pub u64,
    // Minimum number of months with positive cash flow (out of 6)
    required_positive_cash_flow_months: pub u64,
    // Maximum revenue volatility in basis points
    max_revenue_volatility_bps: pub u64,
    // Maximum customer concentration in basis points
    max_customer_concentration_bps: pub u64,
    // Maximum debt-to-revenue ratio in basis points
    max_debt_ratio_bps: pub u64,
    // 1 if lender requires no missed repayments, 0 to skip check
    require_no_missed_repayments: pub u64,
    // Minimum account age in months
    required_account_age_months: pub u64,
) {
    // CHECK 1: Average monthly revenue >= threshold
    let total_revenue: u64 =
        monthly_revenue[0]
        + monthly_revenue[1]
        + monthly_revenue[2]
        + monthly_revenue[3]
        + monthly_revenue[4]
        + monthly_revenue[5];
    let avg_revenue: u64 = total_revenue / 6;
    assert(avg_revenue >= required_monthly_revenue);

    // CHECK 2: Average balance >= threshold
    let total_balance: u64 =
        monthly_balances[0]
        + monthly_balances[1]
        + monthly_balances[2]
        + monthly_balances[3]
        + monthly_balances[4]
        + monthly_balances[5];
    let avg_balance: u64 = total_balance / 6;
    assert(avg_balance >= required_avg_balance);

    // CHECK 3: Positive cash flow months >= required
    let mut positive_months: u64 = 0;
    for i in 0..6 {
        if monthly_revenue[i] > monthly_expenses[i] {
            positive_months += 1;
        }
    }
    assert(positive_months >= required_positive_cash_flow_months);

    // CHECK 4: Revenue volatility <= maximum
    // The prover computes this off-chain and provides as a private input.
    assert(revenue_volatility_bps <= max_revenue_volatility_bps);

    // CHECK 5: Customer concentration <= maximum
    assert(customer_concentration_bps <= max_customer_concentration_bps);

    // CHECK 6: Debt ratio <= maximum
    assert(debt_ratio_bps <= max_debt_ratio_bps);

    // CHECK 7: No missed repayments (if required by lender)
    if require_no_missed_repayments == 1 {
        assert(has_missed_repayments == 0);
    }

    // CHECK 8: Account age >= required
    assert(account_age_months >= required_account_age_months);
}

// Unit tests

#[test]
fn test_passing_borrower() {
    // Healthy business: NGN 6M avg revenue, NGN 1.2M avg balance
    main(
        // 6 months revenue in kobo: NGN 5M to NGN 7.5M
        [500000000, 550000000, 600000000, 650000000, 700000000, 750000000],
        // 6 months expenses in kobo: NGN 3M to NGN 4M
        [300000000, 320000000, 340000000, 360000000, 380000000, 400000000],
        // 6 months balances in kobo: NGN 1M to NGN 1.4M
        [100000000, 110000000, 120000000, 130000000, 140000000, 120000000],
        // volatility 10% = 1000 bps
        1000,
        // customer concentration 20% = 2000 bps
        2000,
        // debt ratio 15% = 1500 bps
        1500,
        // no missed repayments
        0,
        // account age 24 months
        24,
        // lender thresholds
        500000000,  // required revenue: NGN 5M
        50000000,   // required balance: NGN 500k
        4,          // required positive CF months
        1500,       // max volatility: 15%
        2500,       // max customer conc: 25%
        2500,       // max debt ratio: 25%
        1,          // require no missed repayments
        12,         // required account age: 12 months
    );
}

#[test]
fn test_passing_with_no_repayment_check() {
    // Same as above but lender does not require repayment check (require = 0)
    main(
        [500000000, 550000000, 600000000, 650000000, 700000000, 750000000],
        [300000000, 320000000, 340000000, 360000000, 380000000, 400000000],
        [100000000, 110000000, 120000000, 130000000, 140000000, 120000000],
        1000,
        2000,
        1500,
        1,  // has missed repayments -- but lender does not require check
        24,
        500000000,
        50000000,
        4,
        1500,
        2500,
        2500,
        0,  // require_no_missed_repayments = 0 (skip check)
        12,
    );
}

#[test]
fn test_exact_threshold_values() {
    // Metrics exactly at the threshold -- should still pass
    main(
        [500000000, 500000000, 500000000, 500000000, 500000000, 500000000],
        [400000000, 400000000, 400000000, 400000000, 400000000, 400000000],
        [50000000, 50000000, 50000000, 50000000, 50000000, 50000000],
        1500,  // exactly at max
        2500,
        2500,
        0,
        12,  // exactly at min
        500000000,
        50000000,
        4,
        1500,
        2500,
        2500,
        1,
        12,
    );
}
