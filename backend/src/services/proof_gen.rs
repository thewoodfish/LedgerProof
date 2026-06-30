use crate::models::{
    metrics::{FinancialMetrics, LendingPolicy},
    proof::{ProofPackage, ProvenPredicate, PublicInputs},
};
use anyhow::{anyhow, Result};
use std::{
    fs,
    path::Path,
    process::Command,
};
use uuid::Uuid;

/// Generate a UltraHonk ZK proof for the given metrics against a lending policy.
pub fn generate(
    proof_id: Uuid,
    metrics: &FinancialMetrics,
    policy: &LendingPolicy,
    circuits_dir: &str,
) -> Result<ProofPackage> {
    let circuit_path = Path::new(circuits_dir)
        .canonicalize()
        .map_err(|e| anyhow!("circuits_dir '{}' not found: {e}", circuits_dir))?;
    let circuit_path = circuit_path.as_path();

    // Build public and private inputs
    let pub_inputs = public_inputs(policy);
    let prover_toml = build_prover_toml(metrics, &pub_inputs);

    // Write Prover.toml into the circuit directory
    let prover_toml_path = circuit_path.join("Prover.toml");
    fs::write(&prover_toml_path, &prover_toml)
        .map_err(|e| anyhow!("Failed to write Prover.toml: {e}"))?;

    // Step 1: nargo execute — compiles witness
    let nargo_out = Command::new("nargo")
        .args(["execute", "--silence-warnings"])
        .current_dir(circuit_path)
        .output()
        .map_err(|e| anyhow!("Failed to run nargo: {e}"))?;

    if !nargo_out.status.success() {
        let stderr = String::from_utf8_lossy(&nargo_out.stderr);
        return Err(anyhow!("nargo execute failed: {}", stderr));
    }

    // Witness is at target/<package_name>.gz
    let witness_path = circuit_path.join("target/lending.gz");
    if !witness_path.exists() {
        return Err(anyhow!("Witness file not found after nargo execute"));
    }

    // Step 2: Write verification key
    let circuit_json = circuit_path.join("target/lending.json");
    let vk_path = circuit_path.join("target/vk");

    let vk_out = Command::new("bb")
        .args([
            "write_vk",
            "--scheme", "ultra_honk",
            "-b", circuit_json.to_str().unwrap(),
            "-o", vk_path.to_str().unwrap(),
        ])
        .current_dir(circuit_path)
        .output()
        .map_err(|e| anyhow!("Failed to run bb write_vk: {e}"))?;

    if !vk_out.status.success() {
        let stderr = String::from_utf8_lossy(&vk_out.stderr);
        return Err(anyhow!("bb write_vk failed: {}", stderr));
    }

    // Step 3: bb prove — generate UltraHonk proof
    let proof_path = circuit_path.join("target/proof");

    let prove_out = Command::new("bb")
        .args([
            "prove",
            "--scheme", "ultra_honk",
            "-b", circuit_json.to_str().unwrap(),
            "-w", witness_path.to_str().unwrap(),
            "-o", proof_path.to_str().unwrap(),
        ])
        .current_dir(circuit_path)
        .output()
        .map_err(|e| anyhow!("Failed to run bb prove: {e}"))?;

    if !prove_out.status.success() {
        let stderr = String::from_utf8_lossy(&prove_out.stderr);
        return Err(anyhow!("bb prove failed: {}", stderr));
    }

    // bb writes proof to target/proof/proof and public inputs to target/proof/public_inputs
    let proof_bytes = fs::read(proof_path.join("proof"))
        .map_err(|e| anyhow!("Failed to read proof: {e}"))?;
    let vk_bytes = fs::read(vk_path.join("vk"))
        .map_err(|e| anyhow!("Failed to read vk: {e}"))?;
    let pub_inputs_bytes = fs::read(proof_path.join("public_inputs"))
        .unwrap_or_default();

    // Clean up Prover.toml (contains private data)
    let _ = fs::remove_file(&prover_toml_path);

    let predicates = build_predicates(metrics, policy);

    Ok(ProofPackage {
        proof_id,
        merchant_id: metrics.merchant_id,
        circuit_id: "lending_v1".to_string(),
        proof_hex: hex::encode(&proof_bytes),
        vk_hex: hex::encode(&vk_bytes),
        pub_inputs_hex: hex::encode(&pub_inputs_bytes),
        public_inputs: pub_inputs,
        predicates,
        created_at: chrono::Utc::now(),
    })
}

/// Verify a UltraHonk proof from a proof package.
pub fn verify(package: &ProofPackage, circuits_dir: &str) -> Result<bool> {
    let circuit_path = Path::new(circuits_dir)
        .canonicalize()
        .map_err(|e| anyhow!("circuits_dir '{}' not found: {e}", circuits_dir))?;
    let circuit_path = circuit_path.as_path();

    let proof_bytes = hex::decode(&package.proof_hex)
        .map_err(|e| anyhow!("Invalid proof hex: {e}"))?;
    let vk_bytes = hex::decode(&package.vk_hex)
        .map_err(|e| anyhow!("Invalid vk hex: {e}"))?;
    let pub_inputs_bytes = hex::decode(&package.pub_inputs_hex)
        .map_err(|e| anyhow!("Invalid pub_inputs hex: {e}"))?;

    let tmp = tempfile::tempdir()?;
    let proof_path = tmp.path().join("proof");
    let vk_path = tmp.path().join("vk");
    let pub_inputs_path = tmp.path().join("public_inputs");
    fs::write(&proof_path, proof_bytes)?;
    fs::write(&vk_path, vk_bytes)?;
    fs::write(&pub_inputs_path, pub_inputs_bytes)?;

    let out = Command::new("bb")
        .args([
            "verify",
            "--scheme", "ultra_honk",
            "-k", vk_path.to_str().unwrap(),
            "-p", proof_path.to_str().unwrap(),
            "-i", pub_inputs_path.to_str().unwrap(),
        ])
        .current_dir(circuit_path)
        .output()
        .map_err(|e| anyhow!("Failed to run bb verify: {e}"))?;

    Ok(out.status.success())
}

// ── helpers ────────────────────────────────────────────────────────────────

fn public_inputs(policy: &LendingPolicy) -> PublicInputs {
    PublicInputs {
        required_monthly_revenue: policy.required_monthly_revenue.unwrap_or(0) as u64,
        required_avg_balance: policy.required_avg_balance.unwrap_or(0) as u64,
        required_positive_cash_flow_months: policy
            .required_positive_cash_flow_months
            .unwrap_or(0) as u64,
        max_revenue_volatility_bps: policy.max_revenue_volatility_bps.unwrap_or(10_000) as u64,
        max_customer_concentration_bps: policy
            .max_customer_concentration_bps
            .unwrap_or(10_000) as u64,
        max_debt_ratio_bps: policy.max_debt_ratio_bps.unwrap_or(10_000) as u64,
        require_no_missed_repayments: if policy.require_no_missed_repayments == Some(true) {
            1
        } else {
            0
        },
        required_account_age_months: policy.required_account_age_months.unwrap_or(0) as u64,
    }
}

/// Build Prover.toml for the lending Noir circuit.
/// Private inputs contain the actual financial values.
fn build_prover_toml(metrics: &FinancialMetrics, pub_inputs: &PublicInputs) -> String {
    // Collect 6 months of data, padding with zeros if needed
    let mut months: Vec<String> = metrics
        .monthly_revenue
        .as_object()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    months.sort();
    months.truncate(6);
    while months.len() < 6 {
        months.push(format!("0000-{:02}", months.len() + 1));
    }

    let revenue_arr = months_array(
        &months,
        metrics.monthly_revenue.as_object(),
        |v| v.as_i64().unwrap_or(0) as u64,
    );

    let expenses_arr: Vec<u64> = months
        .iter()
        .map(|m| {
            let cf = metrics
                .monthly_cash_flow
                .get(m)
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let rev = metrics
                .monthly_revenue
                .get(m)
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            (rev - cf).max(0) as u64
        })
        .collect();

    let balance_arr = vec![metrics.avg_monthly_balance as u64; 6];

    format!(
        r#"# Private inputs — actual financial values (never revealed)
monthly_revenue = [{revenue}]
monthly_expenses = [{expenses}]
monthly_balances = [{balances}]
revenue_volatility_bps = {vol}
customer_concentration_bps = {cust}
debt_ratio_bps = {debt}
has_missed_repayments = {missed}
account_age_months = {age}

# Public inputs — thresholds set by lender
required_monthly_revenue = {req_rev}
required_avg_balance = {req_bal}
required_positive_cash_flow_months = {req_cf}
max_revenue_volatility_bps = {max_vol}
max_customer_concentration_bps = {max_cust}
max_debt_ratio_bps = {max_debt}
require_no_missed_repayments = {req_missed}
required_account_age_months = {req_age}
"#,
        revenue = join_u64(&revenue_arr),
        expenses = join_u64(&expenses_arr),
        balances = join_u64(&balance_arr),
        vol = metrics.revenue_volatility_bps,
        cust = metrics.customer_concentration_bps,
        debt = metrics.debt_ratio_bps,
        missed = if metrics.has_missed_repayments { 1u64 } else { 0u64 },
        age = metrics.account_age_months,
        req_rev = pub_inputs.required_monthly_revenue,
        req_bal = pub_inputs.required_avg_balance,
        req_cf = pub_inputs.required_positive_cash_flow_months,
        max_vol = pub_inputs.max_revenue_volatility_bps,
        max_cust = pub_inputs.max_customer_concentration_bps,
        max_debt = pub_inputs.max_debt_ratio_bps,
        req_missed = pub_inputs.require_no_missed_repayments,
        req_age = pub_inputs.required_account_age_months,
    )
}

fn months_array(
    months: &[String],
    map: Option<&serde_json::Map<String, serde_json::Value>>,
    extract: impl Fn(&serde_json::Value) -> u64,
) -> Vec<u64> {
    months
        .iter()
        .map(|m| {
            map.and_then(|o| o.get(m))
                .map(|v| extract(v))
                .unwrap_or(0)
        })
        .collect()
}

fn join_u64(v: &[u64]) -> String {
    v.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn build_predicates(metrics: &FinancialMetrics, policy: &LendingPolicy) -> Vec<ProvenPredicate> {
    let (_, failures) = crate::services::metrics::check_policy(metrics, policy);
    let failure_set: std::collections::HashSet<String> = failures.into_iter().collect();

    let mut preds = Vec::new();

    macro_rules! pred {
        ($name:expr, $desc:expr) => {{
            let satisfied = !failure_set.iter().any(|f| f.contains($name));
            preds.push(ProvenPredicate {
                name: $name.to_string(),
                description: $desc.to_string(),
                satisfied,
            });
        }};
    }

    if policy.required_monthly_revenue.is_some() {
        pred!("avg monthly revenue", "Monthly revenue meets minimum threshold");
    }
    if policy.required_avg_balance.is_some() {
        pred!("avg balance", "Average balance meets minimum threshold");
    }
    if policy.required_positive_cash_flow_months.is_some() {
        pred!("positive cash flow months", "Sufficient months of positive cash flow");
    }
    if policy.max_revenue_volatility_bps.is_some() {
        pred!("revenue volatility", "Revenue volatility within acceptable range");
    }
    if policy.max_customer_concentration_bps.is_some() {
        pred!("customer concentration", "No single customer dominates revenue");
    }
    if policy.max_debt_ratio_bps.is_some() {
        pred!("debt ratio", "Debt payments within acceptable ratio of revenue");
    }
    pred!("missed loan repayments", "No missed loan repayments detected");
    if policy.required_account_age_months.is_some() {
        pred!("account age", "Account has sufficient history");
    }

    preds
}
