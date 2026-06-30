-- Users
CREATE TABLE IF NOT EXISTS users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username      TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role          TEXT NOT NULL CHECK (role IN ('borrower', 'lender')),
    full_name     TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Lender profiles (one per lender user, created on registration)
CREATE TABLE IF NOT EXISTS lender_profiles (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    display_name TEXT NOT NULL DEFAULT '',
    description  TEXT NOT NULL DEFAULT '',
    policy       JSONB NOT NULL DEFAULT '{}',
    published    BOOLEAN NOT NULL DEFAULT FALSE,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_lender_profiles_user ON lender_profiles(user_id);

-- Loan applications: borrower → lender
CREATE TABLE IF NOT EXISTS loan_applications (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    borrower_id       UUID NOT NULL REFERENCES users(id),
    lender_profile_id UUID NOT NULL REFERENCES lender_profiles(id),
    metrics_id        UUID NOT NULL REFERENCES financial_metrics(id),
    proof_id          UUID REFERENCES proofs(id),
    status            TEXT NOT NULL DEFAULT 'pending'
                      CHECK (status IN ('pending', 'approved', 'rejected')),
    decision_reason   TEXT,
    amount_requested  BIGINT,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    decided_at        TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_applications_borrower    ON loan_applications(borrower_id);
CREATE INDEX IF NOT EXISTS idx_applications_lender      ON loan_applications(lender_profile_id);
