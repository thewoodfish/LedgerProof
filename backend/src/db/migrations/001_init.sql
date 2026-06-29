CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Statements uploaded by merchants
CREATE TABLE IF NOT EXISTS statements (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL,
    filename    TEXT NOT NULL,
    month       TEXT NOT NULL,   -- e.g. "2024-01"
    raw_text    TEXT,
    status      TEXT NOT NULL DEFAULT 'pending',  -- pending | parsed | error
    error_msg   TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_statements_merchant ON statements(merchant_id);

-- Normalized transactions extracted from statements
CREATE TABLE IF NOT EXISTS transactions (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    statement_id UUID NOT NULL REFERENCES statements(id) ON DELETE CASCADE,
    merchant_id  UUID NOT NULL,
    date         DATE NOT NULL,
    description  TEXT NOT NULL,
    credit       BIGINT NOT NULL DEFAULT 0,   -- kobo
    debit        BIGINT NOT NULL DEFAULT 0,   -- kobo
    balance      BIGINT NOT NULL DEFAULT 0,   -- kobo
    category     TEXT NOT NULL DEFAULT 'unknown',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_transactions_merchant ON transactions(merchant_id);
CREATE INDEX IF NOT EXISTS idx_transactions_statement ON transactions(statement_id);
CREATE INDEX IF NOT EXISTS idx_transactions_date ON transactions(date);

-- Computed financial metrics per merchant
CREATE TABLE IF NOT EXISTS financial_metrics (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id                 UUID NOT NULL,
    computed_at                 TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Revenue
    monthly_revenue             JSONB NOT NULL DEFAULT '{}',  -- {"2024-01": 500000000}
    avg_monthly_revenue         BIGINT NOT NULL DEFAULT 0,    -- kobo
    revenue_volatility_bps      INT NOT NULL DEFAULT 0,       -- basis points

    -- Cash flow
    monthly_cash_flow           JSONB NOT NULL DEFAULT '{}',
    positive_cash_flow_months   INT NOT NULL DEFAULT 0,

    -- Balance
    avg_monthly_balance         BIGINT NOT NULL DEFAULT 0,
    min_balance                 BIGINT NOT NULL DEFAULT 0,

    -- Growth
    revenue_growth_months       INT NOT NULL DEFAULT 0,

    -- Activity
    avg_monthly_tx_count        INT NOT NULL DEFAULT 0,

    -- Concentration
    customer_concentration_bps  INT NOT NULL DEFAULT 0,
    supplier_concentration_bps  INT NOT NULL DEFAULT 0,

    -- Expenses
    expense_variance_bps        INT NOT NULL DEFAULT 0,

    -- Debt
    debt_ratio_bps              INT NOT NULL DEFAULT 0,

    -- Repayments
    has_missed_repayments       BOOLEAN NOT NULL DEFAULT FALSE,

    -- Account
    account_age_months          INT NOT NULL DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_metrics_merchant ON financial_metrics(merchant_id);

-- Generated ZK proofs
CREATE TABLE IF NOT EXISTS proofs (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id    UUID NOT NULL,
    metrics_id     UUID NOT NULL REFERENCES financial_metrics(id),
    circuit_id     TEXT NOT NULL DEFAULT 'lending_v1',
    proof_hex      TEXT NOT NULL,
    vk_hex         TEXT NOT NULL,
    public_inputs  JSONB NOT NULL DEFAULT '{}',
    predicates     JSONB NOT NULL DEFAULT '[]',
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_proofs_merchant ON proofs(merchant_id);

-- Loan applications
CREATE TABLE IF NOT EXISTS loan_applications (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id UUID NOT NULL,
    lender_id   UUID NOT NULL,
    proof_id    UUID NOT NULL REFERENCES proofs(id),
    policy      JSONB NOT NULL DEFAULT '{}',
    decision    TEXT,   -- approved | rejected | pending
    reason      TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    decided_at  TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_loans_merchant ON loan_applications(merchant_id);
CREATE INDEX IF NOT EXISTS idx_loans_lender ON loan_applications(lender_id);
