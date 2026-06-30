use crate::models::metrics::LendingPolicy;
use anyhow::{bail, Result};
use std::process::Command;
use uuid::Uuid;

/// Publish a lender's underwriting policy on Stellar.
/// Returns the Stellar transaction hash on success.
pub fn publish_policy_on_chain(
    lender_id: &str,
    policy: &LendingPolicy,
    contract_id: &str,
    stellar_identity: &str,
    stellar_network: &str,
) -> Result<String> {
    if contract_id.is_empty() {
        bail!("SOROBAN_CONTRACT_ID not configured");
    }

    let policy_json = format!(
        concat!(
            r#"{{ "required_monthly_revenue": {}, "required_avg_balance": {}, "#,
            r#""required_positive_cf_months": {}, "max_revenue_volatility_bps": {}, "#,
            r#""max_customer_concentration_bps": {}, "max_debt_ratio_bps": {}, "#,
            r#""require_no_missed_repayments": {}, "required_account_age_months": {} }}"#,
        ),
        policy.required_monthly_revenue.unwrap_or(0),
        policy.required_avg_balance.unwrap_or(0),
        policy.required_positive_cash_flow_months.unwrap_or(0),
        policy.max_revenue_volatility_bps.unwrap_or(0),
        policy.max_customer_concentration_bps.unwrap_or(0),
        policy.max_debt_ratio_bps.unwrap_or(0),
        if policy.require_no_missed_repayments.unwrap_or(false) { 1u64 } else { 0u64 },
        policy.required_account_age_months.unwrap_or(0),
    );

    tracing::info!(
        "Publishing lender policy on Stellar (lender_id={}, contract={})",
        lender_id, contract_id
    );

    let output = Command::new("stellar")
        .args([
            "contract", "invoke",
            "--network", stellar_network,
            "--source", stellar_identity,
            "--id", contract_id,
            "--",
            "publish_policy",
            "--signer", stellar_identity,
            "--lender_id", lender_id,
            "--policy", &policy_json,
        ])
        .output()?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        tracing::error!("publish_policy failed: {}", stderr);
        bail!("Soroban publish_policy failed: {}", stderr.trim());
    }

    let tx_hash = stderr
        .lines()
        .find(|l| l.contains("Signing transaction:"))
        .and_then(|l| l.split_whitespace().last())
        .map(str::to_string)
        .unwrap_or_else(|| "unknown".to_string());

    tracing::info!(
        "Policy published on Stellar: {} — https://stellar.expert/explorer/{}/tx/{}",
        tx_hash, stellar_network, tx_hash
    );

    Ok(tx_hash)
}

/// Write 8 policy values as 64 big-endian bytes to a temp file.
/// Returns the file path. The stellar CLI accepts Bytes args via file.
fn write_public_inputs_file(p: &LendingPolicy) -> Result<std::path::PathBuf> {
    let values: [u64; 8] = [
        p.required_monthly_revenue.unwrap_or(0) as u64,
        p.required_avg_balance.unwrap_or(0) as u64,
        p.required_positive_cash_flow_months.unwrap_or(0) as u64,
        p.max_revenue_volatility_bps.unwrap_or(0) as u64,
        p.max_customer_concentration_bps.unwrap_or(0) as u64,
        p.max_debt_ratio_bps.unwrap_or(0) as u64,
        if p.require_no_missed_repayments.unwrap_or(false) { 1u64 } else { 0u64 },
        p.required_account_age_months.unwrap_or(0) as u64,
    ];

    let mut buf = [0u8; 64];
    for (i, v) in values.iter().enumerate() {
        buf[i * 8..i * 8 + 8].copy_from_slice(&v.to_be_bytes());
    }

    let path = std::env::temp_dir().join(format!("lp_pub_inputs_{}.bin", Uuid::new_v4()));
    std::fs::write(&path, &buf)?;
    Ok(path)
}

/// Record an approved/rejected loan decision on Stellar testnet.
/// Returns the Stellar transaction hash on success.
pub fn record_decision_on_chain(
    proof_id: Uuid,
    proof_hex: &str,
    policy: &LendingPolicy,
    decision: &str,
    contract_id: &str,
    stellar_identity: &str,
    stellar_network: &str,
) -> Result<String> {
    if contract_id.is_empty() {
        bail!("SOROBAN_CONTRACT_ID not configured");
    }

    // proof_id → 16 raw bytes → 32 hex chars
    let proof_id_hex: String = proof_id
        .as_bytes()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect();

    // proof_hash → first 32 bytes (64 hex chars) of the proof
    let proof_hash_hex = if proof_hex.len() >= 64 {
        proof_hex[..64].to_lowercase()
    } else {
        format!("{:0<64}", proof_hex.to_lowercase())
    };

    // Write public inputs as raw binary (64 bytes)
    let inputs_file = write_public_inputs_file(policy)?;

    let policy_json = format!(
        concat!(
            r#"{{ "required_monthly_revenue": {}, "required_avg_balance": {}, "#,
            r#""required_positive_cf_months": {}, "max_revenue_volatility_bps": {}, "#,
            r#""max_customer_concentration_bps": {}, "max_debt_ratio_bps": {}, "#,
            r#""require_no_missed_repayments": {}, "required_account_age_months": {} }}"#,
        ),
        policy.required_monthly_revenue.unwrap_or(0),
        policy.required_avg_balance.unwrap_or(0),
        policy.required_positive_cash_flow_months.unwrap_or(0),
        policy.max_revenue_volatility_bps.unwrap_or(0),
        policy.max_customer_concentration_bps.unwrap_or(0),
        policy.max_debt_ratio_bps.unwrap_or(0),
        if policy.require_no_missed_repayments.unwrap_or(false) { 1u64 } else { 0u64 },
        policy.required_account_age_months.unwrap_or(0),
    );

    let decision_symbol = decision.to_uppercase();

    tracing::info!(
        "Recording loan decision on Stellar (contract={}, decision={})",
        contract_id,
        decision_symbol
    );

    let output = Command::new("stellar")
        .args([
            "contract", "invoke",
            "--network", stellar_network,
            "--source", stellar_identity,
            "--id", contract_id,
            "--",
            "record_decision",
            "--lender", stellar_identity,
            "--proof_id", &proof_id_hex,
            "--proof_hash", &proof_hash_hex,
            "--public_inputs_bytes-file-path", inputs_file.to_str().unwrap_or("/tmp/lp_inputs.bin"),
            "--policy", &policy_json,
            "--decision", &decision_symbol,
        ])
        .output();

    // Clean up temp file regardless of outcome
    let _ = std::fs::remove_file(&inputs_file);

    let output = output?;
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        tracing::error!("Soroban invoke failed: {}", stderr);
        bail!("Soroban invocation failed: {}", stderr.trim());
    }

    // Extract signing tx hash from stderr (stellar CLI always logs it there)
    let tx_hash = stderr
        .lines()
        .find(|l| l.contains("Signing transaction:"))
        .and_then(|l| l.split_whitespace().last())
        .map(str::to_string)
        .unwrap_or_else(|| "unknown".to_string());

    tracing::info!("Soroban tx recorded: {} — https://stellar.expert/explorer/{}/tx/{}", tx_hash, stellar_network, tx_hash);

    Ok(tx_hash)
}
