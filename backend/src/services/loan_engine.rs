use crate::models::{
    metrics::LendingPolicy,
    proof::{LoanDecision, ProofPackage},
};
use crate::services::{proof_gen, soroban};
use anyhow::Result;
use uuid::Uuid;

pub struct SorobanConfig<'a> {
    pub contract_id: &'a str,
    pub identity: &'a str,
    pub network: &'a str,
}

/// Verify a proof package, evaluate the loan decision, and record it on Stellar.
/// If approved and the borrower has a Stellar address, XLM is disbursed
/// automatically via the Soroban contract.
pub fn evaluate(
    application_id: Uuid,
    package: &ProofPackage,
    policy: &LendingPolicy,
    circuits_dir: &str,
    lender_id: &str,
    borrower_stellar_address: Option<&str>,
    soroban: &SorobanConfig,
) -> Result<LoanDecision> {
    // 1. Cryptographic proof verification (off-chain, bb verify)
    let proof_verified = match proof_gen::verify(package, circuits_dir) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Proof verification error: {}", e);
            false
        }
    };

    if !proof_verified {
        return Ok(LoanDecision {
            application_id,
            decision: "rejected".to_string(),
            reason: "Cryptographic proof verification failed.".to_string(),
            proof_verified: false,
            policy_met: false,
            failed_predicates: vec!["proof_invalid".to_string()],
            stellar_tx_hash: None,
            disbursement_tx_hash: None,
            disbursement_amount_stroops: None,
        });
    }

    // 2. Policy check against proven predicates
    let failed: Vec<String> = package
        .predicates
        .iter()
        .filter(|p| !p.satisfied)
        .map(|p| p.name.clone())
        .collect();

    let policy_met = failed.is_empty();

    let (decision, reason) = if policy_met {
        (
            "approved".to_string(),
            "All lending criteria verified by ZK proofs.".to_string(),
        )
    } else {
        (
            "rejected".to_string(),
            format!("The following criteria were not met: {}", failed.join(", ")),
        )
    };

    // 3. Record decision on Stellar — contract auto-disburses XLM if APPROVED
    //    and lender has configured a loan amount and borrower has a Stellar address.
    let stellar_tx_hash = match soroban::record_decision_on_chain(
        package.proof_id,
        &package.proof_hex,
        policy,
        &decision,
        lender_id,
        borrower_stellar_address,
        soroban.contract_id,
        soroban.identity,
        soroban.network,
    ) {
        Ok(hash) => {
            tracing::info!("Decision recorded on Stellar: {}", hash);
            Some(hash)
        }
        Err(e) => {
            tracing::error!("Failed to record on Stellar (non-fatal): {}", e);
            None
        }
    };

    // The disbursement tx IS the same Stellar tx — the contract disburses atomically
    // within record_decision. We surface it separately in the UI for clarity.
    let (disbursement_tx_hash, disbursement_amount_stroops) =
        if decision == "approved" && stellar_tx_hash.is_some() && borrower_stellar_address.is_some() {
            (stellar_tx_hash.clone(), None::<i64>) // amount fetched from contract config
        } else {
            (None, None)
        };

    Ok(LoanDecision {
        application_id,
        decision,
        reason,
        proof_verified,
        policy_met,
        failed_predicates: failed,
        stellar_tx_hash,
        disbursement_tx_hash,
        disbursement_amount_stroops,
    })
}
