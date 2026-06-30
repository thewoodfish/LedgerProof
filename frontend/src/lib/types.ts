// ── Core domain ───────────────────────────────────────────────────────────

export interface User {
  id: string;
  username: string;
  role: "borrower" | "lender";
  full_name: string | null;
}

export interface AuthResponse {
  token: string;
  user: User;
}

// ── Transactions & statements ──────────────────────────────────────────────

export interface Transaction {
  id: string;
  statement_id: string;
  merchant_id: string;
  date: string;
  description: string;
  credit: number;  // kobo
  debit: number;   // kobo
  balance: number; // kobo
  category: string;
  created_at: string;
}

// ── Metrics ────────────────────────────────────────────────────────────────

export interface MetricsSummary {
  metrics_id: string;
  merchant_id: string;
  summary: {
    avg_monthly_revenue_naira: number;
    avg_monthly_balance_naira: number;
    positive_cash_flow_months: number;
    revenue_volatility_pct: number;
    debt_ratio_pct: number;
    customer_concentration_pct: number;
    has_missed_repayments: boolean;
    account_age_months: number;
    revenue_growth_months: number;
    avg_monthly_tx_count: number;
  };
  monthly_revenue: Record<string, number>;
  monthly_cash_flow: Record<string, number>;
}

// ── ZK Proofs ──────────────────────────────────────────────────────────────

export interface ProvenPredicate {
  name: string;
  description: string;
  satisfied: boolean;
}

export interface PublicInputs {
  required_monthly_revenue: number;
  required_avg_balance: number;
  required_positive_cash_flow_months: number;
  max_revenue_volatility_bps: number;
  max_customer_concentration_bps: number;
  max_debt_ratio_bps: number;
  require_no_missed_repayments: number;
  required_account_age_months: number;
}

export interface ProofPackage {
  proof_id: string;
  merchant_id: string;
  circuit_id: string;
  proof_hex: string;
  vk_hex: string;
  pub_inputs_hex: string;
  public_inputs: PublicInputs;
  predicates: ProvenPredicate[];
  created_at: string;
}

// ── Lending policy ─────────────────────────────────────────────────────────

export interface LendingPolicy {
  required_monthly_revenue?: number;
  required_avg_balance?: number;
  required_positive_cash_flow_months?: number;
  max_revenue_volatility_bps?: number;
  max_customer_concentration_bps?: number;
  max_debt_ratio_bps?: number;
  require_no_missed_repayments?: boolean;
  required_account_age_months?: number;
}

// ── Stellar ────────────────────────────────────────────────────────────────

export interface StellarRecord {
  tx_hash: string | null;
  explorer_url: string | null;
  contract_id: string;
  network: string;
}

// ── Lender profiles ────────────────────────────────────────────────────────

export interface LenderProfile {
  id: string;
  user_id: string;
  display_name: string;
  description: string;
  policy: LendingPolicy;
  published: boolean;
  stellar_policy_tx: string | null;
  stellar?: StellarRecord;
  created_at: string;
  updated_at: string;
}

// ── Applications ───────────────────────────────────────────────────────────

export interface LoanApplication {
  id: string;
  status: "pending" | "approved" | "rejected";
  decision_reason: string | null;
  amount_requested: number | null;
  created_at: string;
  decided_at: string | null;
  // borrower view
  lender?: { display_name: string };
  // lender view
  borrower_ref?: string;
}

export interface ProofDetail {
  id: string;
  circuit_id: string;
  proof_hash: string;
  vk_hash: string;
  proof_size_bytes: number;
  pub_inputs_hex: string;
  public_inputs: PublicInputs;
}

export interface VerifyResult {
  application_id: string;
  status: string;
  decision_reason: string;
  proof_verified: boolean;
  predicates: ProvenPredicate[];
  stellar?: StellarRecord;
  proof?: ProofDetail;
}
