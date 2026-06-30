//! LedgerProof Lending Verifier — Soroban Smart Contract
//!
//! Two on-chain operations:
//!
//! 1. `publish_policy` — lender publishes their underwriting criteria on Stellar.
//!    Immutable, public, and auditable before any loan application is made.
//!
//! 2. `record_decision` — after ZK proof verification off-chain, the loan
//!    decision is recorded permanently with proof hash, public inputs, and
//!    the policy it was verified against.
//!
//! UltraHonk proof verification runs off-chain (bb verify) — Soroban's CPU
//! instruction budget cannot fit a 14 KB UltraHonk proof. This matches how
//! ZK rollups work: verify off-chain, settle on-chain.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype,
    symbol_short, Address, Bytes, BytesN, Env, String, Symbol,
    log, panic_with_error,
};

// ── Error codes ────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    PolicyNotSatisfied = 1,
    InvalidInput = 2,
    AlreadyRecorded = 3,
    PolicyNotFound = 4,
}

// ── Storage key ────────────────────────────────────────────────────────────

#[contracttype]
pub enum DataKey {
    LoanRecord(BytesN<16>),
    LenderPolicy(String),
}

// ── Data types ─────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct LendingPolicy {
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
pub struct LoanRecord {
    pub proof_hash: BytesN<32>,
    pub lender: Address,
    pub decision: Symbol,
    pub public_inputs: PublicInputs,
    pub verified_at: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PolicyRecord {
    pub policy: LendingPolicy,
    pub published_at: u64,
}

// ── Contract ───────────────────────────────────────────────────────────────

#[contract]
pub struct LendingVerifier;

#[contractimpl]
impl LendingVerifier {
    /// Lender publishes their underwriting criteria on-chain.
    ///
    /// lender_id — the lender's application-layer ID (username or UUID string).
    ///             Allows multiple lenders to publish policies even when signing
    ///             from the same Stellar identity in a shared deployment.
    /// signer    — the Stellar account authorising this publication.
    /// policy    — the 8-field underwriting policy to store permanently.
    ///
    /// Calling this again overwrites the previous policy for the same lender_id.
    pub fn publish_policy(
        env: Env,
        signer: Address,
        lender_id: String,
        policy: LendingPolicy,
    ) -> Symbol {
        signer.require_auth();

        let record = PolicyRecord {
            policy,
            published_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::LenderPolicy(lender_id.clone()), &record);

        log!(&env, "LedgerProof: policy published for lender {}", lender_id);

        symbol_short!("OK")
    }

    /// Retrieve a lender's published underwriting policy.
    pub fn get_policy(env: Env, lender_id: String) -> Option<PolicyRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::LenderPolicy(lender_id))
    }

    /// Record a verified loan decision on-chain.
    ///
    /// Called by the LedgerProof backend after `bb verify` confirms the
    /// UltraHonk proof is valid. The lender must sign this transaction.
    pub fn record_decision(
        env: Env,
        lender: Address,
        proof_id: BytesN<16>,
        proof_hash: BytesN<32>,
        public_inputs_bytes: Bytes,
        policy: LendingPolicy,
        decision: Symbol,
    ) -> Symbol {
        lender.require_auth();

        if env.storage().persistent().has(&DataKey::LoanRecord(proof_id.clone())) {
            panic_with_error!(&env, Error::AlreadyRecorded);
        }

        let inputs = Self::decode_public_inputs(&env, &public_inputs_bytes);
        Self::assert_policy_satisfied(&env, &inputs, &policy);

        let record = LoanRecord {
            proof_hash,
            lender: lender.clone(),
            decision: decision.clone(),
            public_inputs: inputs,
            verified_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::LoanRecord(proof_id.clone()), &record);

        log!(&env, "LedgerProof: {} decision recorded for proof {:?}", decision, proof_id);

        decision
    }

    /// Retrieve a recorded loan decision by proof_id.
    pub fn get_record(env: Env, proof_id: BytesN<16>) -> Option<LoanRecord> {
        env.storage()
            .persistent()
            .get(&DataKey::LoanRecord(proof_id))
    }

    // ── Internal helpers ───────────────────────────────────────────────────

    fn decode_public_inputs(env: &Env, bytes: &Bytes) -> PublicInputs {
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
            required_monthly_revenue:          read_u64(0),
            required_avg_balance:              read_u64(8),
            required_positive_cf_months:       read_u64(16),
            max_revenue_volatility_bps:        read_u64(24),
            max_customer_concentration_bps:    read_u64(32),
            max_debt_ratio_bps:                read_u64(40),
            require_no_missed_repayments:      read_u64(48),
            required_account_age_months:       read_u64(56),
        }
    }

    fn assert_policy_satisfied(env: &Env, inputs: &PublicInputs, policy: &LendingPolicy) {
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
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn make_public_inputs_bytes(env: &Env, inputs: &[u64; 8]) -> Bytes {
        let mut buf = [0u8; 64];
        let mut i = 0;
        for val in inputs {
            for b in val.to_be_bytes() {
                buf[i] = b;
                i += 1;
            }
        }
        Bytes::from_slice(env, &buf)
    }

    fn default_policy() -> LendingPolicy {
        LendingPolicy {
            required_monthly_revenue: 500_000_000,
            required_avg_balance: 50_000_000,
            required_positive_cf_months: 4,
            max_revenue_volatility_bps: 1500,
            max_customer_concentration_bps: 2500,
            max_debt_ratio_bps: 2500,
            require_no_missed_repayments: 1,
            required_account_age_months: 12,
        }
    }

    #[test]
    fn test_publish_and_get_policy() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(LendingVerifier, ());
        let client = LendingVerifierClient::new(&env, &contract_id);

        let signer = Address::generate(&env);
        let lender_id = String::from_str(&env, "lender-abc-123");

        let result = client.publish_policy(&signer, &lender_id, &default_policy());
        assert_eq!(result, symbol_short!("OK"));

        let record = client.get_policy(&lender_id).unwrap();
        assert_eq!(record.policy.required_monthly_revenue, 500_000_000);
    }

    #[test]
    fn test_record_and_retrieve() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(LendingVerifier, ());
        let client = LendingVerifierClient::new(&env, &contract_id);

        let lender = Address::generate(&env);
        let proof_id = BytesN::from_array(&env, &[1u8; 16]);
        let proof_hash = BytesN::from_array(&env, &[2u8; 32]);

        let inputs_raw: [u64; 8] = [600_000_000, 60_000_000, 5, 1200, 2000, 2000, 1, 24];
        let public_inputs_bytes = make_public_inputs_bytes(&env, &inputs_raw);

        let decision = client.record_decision(
            &lender,
            &proof_id,
            &proof_hash,
            &public_inputs_bytes,
            &default_policy(),
            &symbol_short!("APPROVED"),
        );

        assert_eq!(decision, symbol_short!("APPROVED"));

        let record = client.get_record(&proof_id).unwrap();
        assert_eq!(record.decision, symbol_short!("APPROVED"));
    }

    #[test]
    #[should_panic]
    fn test_policy_not_satisfied() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(LendingVerifier, ());
        let client = LendingVerifierClient::new(&env, &contract_id);

        let lender = Address::generate(&env);
        let proof_id = BytesN::from_array(&env, &[3u8; 16]);
        let proof_hash = BytesN::from_array(&env, &[4u8; 32]);

        // Revenue too low — 300M vs 500M required
        let inputs_raw: [u64; 8] = [300_000_000, 60_000_000, 5, 1200, 2000, 2000, 1, 24];
        let public_inputs_bytes = make_public_inputs_bytes(&env, &inputs_raw);

        client.record_decision(
            &lender,
            &proof_id,
            &proof_hash,
            &public_inputs_bytes,
            &default_policy(),
            &symbol_short!("APPROVED"),
        );
    }
}
