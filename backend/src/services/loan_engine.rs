use crate::models::{
    metrics::LendingPolicy,
    proof::{LoanDecision, ProofPackage},
};
use crate::services::proof_gen;
use anyhow::Result;
use uuid::Uuid;

/// Verify a proof package and evaluate the loan decision.
pub fn evaluate(
    application_id: Uuid,
    package: &ProofPackage,
    _policy: &LendingPolicy,
    circuits_dir: &str,
) -> Result<LoanDecision> {
    // Cryptographic proof verification
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
        });
    }

    // Policy check against proven predicates
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
            format!(
                "The following criteria were not met: {}",
                failed.join(", ")
            ),
        )
    };

    Ok(LoanDecision {
        application_id,
        decision,
        reason,
        proof_verified,
        policy_met,
        failed_predicates: failed,
    })
}
