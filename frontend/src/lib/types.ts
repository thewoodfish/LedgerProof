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
  public_inputs: PublicInputs;
  predicates: ProvenPredicate[];
  created_at: string;
}

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

export interface LoanDecision {
  application_id: string;
  decision: "approved" | "rejected" | "pending";
  reason: string;
  proof_verified: boolean;
  policy_met: boolean;
  failed_predicates: string[];
}
