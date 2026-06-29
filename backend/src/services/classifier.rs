use crate::models::transaction::Category;

/// Keyword-based transaction classifier.
/// Fast deterministic pass — no LLM needed for common patterns.
pub fn classify(description: &str, credit: i64, debit: i64) -> Category {
    let desc = description.to_lowercase();

    // Loan repayment identifiers
    if contains_any(&desc, &[
        "branch", "carbon", "fairmoney", "moniepoint loan", "palmcredit",
        "renmoney", "loan repay", "loan payment", "loan deduction",
        "creditville", "quickcheck", "alat loan", "accessmore",
    ]) {
        return Category::LoanRepayment;
    }

    // Tax
    if contains_any(&desc, &["firs", "lirs", "tax", "vat ", "withholding"]) {
        return Category::Tax;
    }

    // Cash withdrawals
    if contains_any(&desc, &["atm ", "cash withdrawal", "atm withdrawal", "pos cash"]) {
        return Category::CashWithdrawal;
    }

    // Internal transfers
    if contains_any(&desc, &[
        "transfer to self", "own account", "interbank transfer",
        "intra-bank", "nip transfer",
    ]) {
        return Category::Transfer;
    }

    // Revenue signals (credit-only)
    if credit > 0 && debit == 0 {
        if contains_any(&desc, &[
            "pos ", "card payment", "payment received", "transfer from",
            "credit alert", "settlement", "flutterwave", "paystack",
            "stripe", "invoice", "remittance", "sales",
        ]) {
            return Category::Revenue;
        }
        // Any credit with business-sounding keywords
        return Category::Revenue;
    }

    // Expense signals (debit-only)
    if debit > 0 && credit == 0 {
        if contains_any(&desc, &[
            "salary", "wages", "supplier", "vendor", "electricity",
            "nepa", "phcn", "internet", "airtime", "data sub",
            "rent", "insurance", "maintenance", "diesel", "fuel",
            "dstv", "gotv", "subscription", "utilities",
        ]) {
            return Category::Expense;
        }
        return Category::Expense;
    }

    Category::Unknown
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}
