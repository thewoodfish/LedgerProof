//! LedgerProof Lending Verifier — Soroban Smart Contract
//!
//! Responsibilities:
//! 1. Verify a UltraHonk ZK proof submitted by the lending frontend
//! 2. Evaluate the lender's on-chain policy against the proven public inputs
//! 3. Record the loan decision on-chain
//!
//! The UltraHonk verification is delegated to the deployed
//! `indextree/ultrahonk_soroban_contract` verifier. This contract calls
//! that verifier cross-contract and interprets the result.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype,
    Address, Bytes, BytesN, Env, IntoVal, Symbol,
    log, panic_with_error,
};

// ── Error codes ────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    ProofVerificationFailed = 1,
    PolicyNotSatisfied = 2,
    InvalidInput = 3,
    Unauthorized = 4,
    AlreadyInitialized = 5,
}

// ── Storage types ──────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct LendingPolicy {
    /// Minimum average monthly revenue (kobo)
    pub required_monthly_revenue: u64,
    /// Minimum average balance (kobo)
    pub required_avg_balance: u64,
    /// Minimum positive cash-flow months (0–6)
    pub required_positive_cf_months: u64,
    /// Maximum revenue volatility in basis points
    pub max_revenue_volatility_bps: u64,
    /// Maximum customer concentration in basis points
    pub max_customer_concentration_bps: u64,
    /// Maximum debt ratio in basis points
    pub max_debt_ratio_bps: u64,
    /// 1 = require no missed repayments
    pub require_no_missed_repayments: u64,
    /// Minimum account age in months
    pub required_account_age_months: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PublicInputs {
    pub required_monthly_revenue: u64,
    pub required_avg_balance: u64,
    pub required_positive_cf_months: u64,
    pub max_revenue_volatility_bps: u64,
    pub max_customer_concentration_bps: u64,
    pub max_debt_ratio_bps: u64,
    pub require_no_missed_repayments: u64,
    pub required_account_age_months: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct LoanApplication {
    pub merchant_id: BytesN<32>,
    pub proof_id: BytesN<32>,
    pub lender: Address,
    pub decision: Symbol,
    pub verified_at: u64,
}

// ── Storage keys ──────────────────────────────────────────────────────────

const VERIFIER_KEY: Symbol = Symbol::short("VERIFIER");
const ADMIN_KEY: Symbol = Symbol::short("ADMIN");

// ── Contract ───────────────────────────────────────────────────────────────

#[contract]
pub struct LendingVerifier;

#[contractimpl]
impl LendingVerifier {
    // ── Admin ────────────────────────────────────────────────────────────

    /// Initialize the contract with the UltraHonk verifier contract address.
    /// Can only be called once.
    pub fn initialize(env: Env, admin: Address, verifier_contract: Address) {
        if env.storage().instance().has(&ADMIN_KEY) {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&VERIFIER_KEY, &verifier_contract);
    }

    // ── Core loan evaluation ──────────────────────────────────────────────

    /// Evaluate a loan application.
    ///
    /// Steps:
    /// 1. Verify the UltraHonk proof against the circuit VK
    /// 2. Assert the proven public inputs satisfy the lender's on-chain policy
    /// 3. Record the decision in contract storage
    /// 4. Return the decision symbol: `APPROVED` or `REJECTED`
    pub fn evaluate_loan(
        env: Env,
        lender: Address,
        merchant_id: BytesN<32>,
        proof_id: BytesN<32>,
        proof: Bytes,
        vk: Bytes,
        public_inputs_bytes: Bytes,
        policy: LendingPolicy,
    ) -> Symbol {
        lender.require_auth();

        // ── 1. Verify ZK proof ──────────────────────────────────────────
        let verified = Self::verify_ultrahonk(&env, &proof, &vk, &public_inputs_bytes);
        if !verified {
            log!(&env, "ZK proof verification failed for merchant {}", merchant_id);
            // Record rejection
            Self::record_decision(
                &env,
                &merchant_id,
                &proof_id,
                &lender,
                Symbol::new(&env, "REJECTED"),
            );
            panic_with_error!(&env, Error::ProofVerificationFailed);
        }

        // ── 2. Decode public inputs and check policy ──────────────────
        // Public inputs are ABI-encoded as 8 × u64 (big-endian, 8 bytes each)
        let inputs = Self::decode_public_inputs(&env, &public_inputs_bytes);

        // The circuit public inputs MUST match or be stricter than the lender's policy.
        // This prevents a merchant reusing a proof generated under a laxer policy.
        Self::assert_policy_satisfied(&env, &inputs, &policy);

        // ── 3. Record approval ────────────────────────────────────────
        Self::record_decision(
            &env,
            &merchant_id,
            &proof_id,
            &lender,
            Symbol::new(&env, "APPROVED"),
        );

        log!(&env, "Loan APPROVED for merchant {}", merchant_id);
        Symbol::new(&env, "APPROVED")
    }

    /// Retrieve a recorded loan decision.
    pub fn get_decision(env: Env, proof_id: BytesN<32>) -> Option<LoanApplication> {
        env.storage().persistent().get(&proof_id)
    }

    // ── Internals ─────────────────────────────────────────────────────────

    fn verify_ultrahonk(env: &Env, proof: &Bytes, vk: &Bytes, public_inputs: &Bytes) -> bool {
        // Cross-contract call to the deployed UltraHonk verifier.
        // The verifier contract (indextree/ultrahonk_soroban_contract) exposes:
        //   fn verify(proof: Bytes, vk: Bytes, public_inputs: Bytes) -> bool
        let verifier: Address = env
            .storage()
            .instance()
            .get(&VERIFIER_KEY)
            .expect("verifier not set");

        let result: bool = env.invoke_contract(
            &verifier,
            &Symbol::new(env, "verify"),
            soroban_sdk::vec![
                env,
                proof.into_val(env),
                vk.into_val(env),
                public_inputs.into_val(env),
            ],
        );

        result
    }

    fn decode_public_inputs(env: &Env, bytes: &Bytes) -> PublicInputs {
        // Each u64 is stored as 8 big-endian bytes → 8 values = 64 bytes minimum
        if bytes.len() < 64 {
            panic_with_error!(env, Error::InvalidInput);
        }

        let read_u64 = |offset: u32| -> u64 {
            let mut buf = [0u8; 8];
            for i in 0..8u32 {
                buf[i as usize] = bytes.get(offset + i).unwrap_or(0);
            }
            u64::from_be_bytes(buf)
        };

        PublicInputs {
            required_monthly_revenue:     read_u64(0),
            required_avg_balance:         read_u64(8),
            required_positive_cf_months:  read_u64(16),
            max_revenue_volatility_bps:   read_u64(24),
            max_customer_concentration_bps: read_u64(32),
            max_debt_ratio_bps:           read_u64(40),
            require_no_missed_repayments: read_u64(48),
            required_account_age_months:  read_u64(56),
        }
    }

    fn assert_policy_satisfied(env: &Env, inputs: &PublicInputs, policy: &LendingPolicy) {
        // The proven thresholds in the proof must be AT LEAST as strict as what the lender requires.
        // If a proof was generated for a laxer policy, it cannot be reused here.
        let ok = inputs.required_monthly_revenue >= policy.required_monthly_revenue
            && inputs.required_avg_balance >= policy.required_avg_balance
            && inputs.required_positive_cf_months >= policy.required_positive_cf_months
            && inputs.max_revenue_volatility_bps <= policy.max_revenue_volatility_bps
            && inputs.max_customer_concentration_bps <= policy.max_customer_concentration_bps
            && inputs.max_debt_ratio_bps <= policy.max_debt_ratio_bps
            && (policy.require_no_missed_repayments == 0
                || inputs.require_no_missed_repayments == 1)
            && inputs.required_account_age_months >= policy.required_account_age_months;

        if !ok {
            panic_with_error!(env, Error::PolicyNotSatisfied);
        }
    }

    fn record_decision(
        env: &Env,
        merchant_id: &BytesN<32>,
        proof_id: &BytesN<32>,
        lender: &Address,
        decision: Symbol,
    ) {
        let app = LoanApplication {
            merchant_id: merchant_id.clone(),
            proof_id: proof_id.clone(),
            lender: lender.clone(),
            decision,
            verified_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(proof_id, &app);
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_policy_satisfied() {
        let env = Env::default();

        let inputs = PublicInputs {
            required_monthly_revenue: 600_000_000,     // ₦6M (stricter than ₦5M policy)
            required_avg_balance: 60_000_000,          // ₦600k (stricter than ₦500k)
            required_positive_cf_months: 5,
            max_revenue_volatility_bps: 1200,          // 12% (stricter than 15% max)
            max_customer_concentration_bps: 2000,
            max_debt_ratio_bps: 2000,
            require_no_missed_repayments: 1,
            required_account_age_months: 24,
        };

        let policy = LendingPolicy {
            required_monthly_revenue: 500_000_000,
            required_avg_balance: 50_000_000,
            required_positive_cf_months: 4,
            max_revenue_volatility_bps: 1500,
            max_customer_concentration_bps: 2500,
            max_debt_ratio_bps: 2500,
            require_no_missed_repayments: 1,
            required_account_age_months: 12,
        };

        // Should not panic
        LendingVerifier::assert_policy_satisfied(&env, &inputs, &policy);
    }

    #[test]
    #[should_panic]
    fn test_policy_not_satisfied_low_revenue() {
        let env = Env::default();

        let inputs = PublicInputs {
            required_monthly_revenue: 300_000_000,   // ₦3M — below ₦5M policy
            required_avg_balance: 60_000_000,
            required_positive_cf_months: 5,
            max_revenue_volatility_bps: 1200,
            max_customer_concentration_bps: 2000,
            max_debt_ratio_bps: 2000,
            require_no_missed_repayments: 1,
            required_account_age_months: 24,
        };

        let policy = LendingPolicy {
            required_monthly_revenue: 500_000_000,
            required_avg_balance: 50_000_000,
            required_positive_cf_months: 4,
            max_revenue_volatility_bps: 1500,
            max_customer_concentration_bps: 2500,
            max_debt_ratio_bps: 2500,
            require_no_missed_repayments: 1,
            required_account_age_months: 12,
        };

        LendingVerifier::assert_policy_satisfied(&env, &inputs, &policy);
    }
}
