# LedgerProof — System Overview

## What It Does

LedgerProof lets an SME prove it meets a lender's financial criteria without handing over a single bank statement.

The merchant uploads PDFs once. The system extracts transactions, computes 14 financial metrics, and feeds those metrics into a zero-knowledge circuit. The circuit produces a cryptographic proof — a small blob of bytes that a lender can verify mathematically. The lender learns only whether the thresholds are satisfied. The actual numbers never leave the merchant.

---

## How It Works

### 1. Statement Ingestion
The merchant uploads monthly bank statement PDFs via the merchant dashboard. Each file is tagged with a month (e.g. `2024-01`). The backend stores a record and immediately kicks off parsing in a background task.

### 2. Parsing
The parser runs in two stages:
- **lopdf** extracts raw text from the PDF.
- If the extracted text is too short (scanned or image-based PDF), the raw bytes are sent to **GPT-4o** with a vision prompt.
- Either way, GPT-4o receives the text and returns a structured JSON array of transactions: `date`, `description`, `credit`, `debit`, `balance`.

All amounts are stored internally in **kobo** (1 NGN = 100 kobo) to avoid floating-point precision issues.

### 3. Classification
Each transaction is classified by a keyword-based classifier into one of:
`revenue`, `expense`, `loan_repayment`, `transfer`, `cash_withdrawal`, `tax`, `unknown`.

This runs deterministically — no LLM call needed for classification.

### 4. Financial Metrics Engine
All 14 metrics are computed in pure Rust from the classified transactions:

| # | Metric | How |
|---|---|---|
| 1 | Monthly Revenue | Sum of revenue credits per month |
| 2 | Revenue Stability | Coefficient of variation across months (in basis points) |
| 3 | Positive Cash Flow | Count of months where revenue > expenses |
| 4 | Average Monthly Balance | Mean of closing balances |
| 5 | Minimum Cash Reserve | Lowest balance observed |
| 6 | Revenue Growth | Longest streak of month-over-month growth |
| 7 | Business Activity | Average monthly incoming transaction count |
| 8 | Customer Diversity | Largest single counterparty as % of revenue |
| 9 | Supplier Diversity | Largest single counterparty as % of expenses |
| 10 | Expense Stability | Coefficient of variation of monthly expenses |
| 11 | Debt Ratio | Total loan repayments as % of total revenue |
| 12 | Loan Repayment History | Gap detection in repayment months |
| 13 | Account Age | Span from first to last transaction in months |
| 14 | Transaction Frequency | Average count of transactions per month |

### 5. Zero-Knowledge Proof Generation
The metrics are fed into a **Noir** circuit (`circuits/lending/src/main.nr`) via `Prover.toml`. The circuit takes:

- **Private inputs** — the actual financial values (revenue, expenses, balances, pre-computed ratios)
- **Public inputs** — the lender's thresholds (minimum revenue, max volatility, etc.)

The circuit asserts all conditions hold. If any assertion fails, no valid proof can be produced.

The backend shells out to:
```
nargo execute        → generates the witness
bb write_vk          → writes the verification key
bb prove --scheme ultra_honk   → generates the UltraHonk proof
```

The result is a **proof package** — a JSON blob containing the proof bytes (hex), verification key, and public inputs. No financial values are included.

### 6. Proof Verification & Loan Decision
The lender pastes the proof package into the lender dashboard. The backend shells to:
```
bb verify --scheme ultra_honk -k vk -p proof -i public_inputs
```

If the proof is cryptographically valid, the backend checks that the proven thresholds are at least as strict as the lender's current policy. If both pass → **Approved**. If either fails → **Rejected**.

### 7. Soroban Smart Contract (On-chain path)
`contracts/lending_verifier/` is a Soroban (Stellar) contract that:
- Cross-calls the deployed `indextree/ultrahonk_soroban_contract` verifier
- Checks that the proven public inputs satisfy the lender's on-chain policy
- Records the loan decision on-chain with a timestamp

For the hackathon demo the off-chain verification path (step 6) is the primary flow. The Soroban contract is the production path for when the verifier is deployed to Stellar testnet.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Backend API | Rust, Axum, SQLx, PostgreSQL |
| Statement parsing | lopdf (text extraction) + GPT-4o (structuring) |
| Transaction classification | Keyword rules (pure Rust) |
| Financial metrics | Pure Rust, deterministic |
| ZK circuit | Noir 1.0.0-beta.9 |
| Proving backend | Barretenberg UltraHonk (bb 0.87.0) |
| Smart contract | Soroban (Rust), Stellar |
| Frontend | Next.js 15, Tailwind CSS, shadcn-style UI |

---

## Strengths

**Privacy is real, not claimed.**
The lender never sees revenue figures, customer names, balances, or any raw data. The ZK proof is mathematically binding — you cannot construct a valid proof for metrics that don't satisfy the circuit's constraints.

**Reusable proofs.**
A merchant generates one proof package and can share it with any number of lenders. No re-uploading statements per lender.

**Deterministic metrics.**
All 14 metrics are computed in pure Rust with integer arithmetic (kobo). The same inputs always produce the same outputs. No ambiguity between what the prover computed and what the circuit checks.

**Bank-agnostic parsing.**
Because GPT-4o handles the extraction step, the parser works across any Nigerian bank's statement format without writing per-bank rules.

**Modular circuit.**
Each lending predicate maps to a named check in `main.nr`. Adding a new criterion means adding one `assert()` line and one new input pair. The circuit compiles and proves independently of the backend.

**Fast lender integration.**
The lender only needs to call `POST /loan/evaluate` with a JSON blob. No SDK, no statement handling, no financial data storage.

---

## Weaknesses

**Proof generation is slow (~30s).**
Running `nargo execute` + `bb prove` serially on the server takes 20–40 seconds for a single proof. This is acceptable for a hackathon demo but needs parallelisation or a dedicated proving service for production scale.

**Scanned PDFs are lossy.**
When lopdf can't extract text, the PDF bytes are sent to GPT-4o as a base64 image. This only works if the PDF renders to a legible image. Heavily compressed scans or multi-page PDFs over the API token limit will fail or truncate.

**GPT-4o extraction can hallucinate.**
LLM-based parsing is probabilistic. On ambiguous statement layouts the model might miss transactions, merge rows, or misread amounts. There is currently no validation step that cross-checks the parsed total against the stated closing balance.

**Off-chain proof trust.**
The Noir circuit trusts certain inputs (volatility, concentration, debt ratio) that are pre-computed off-chain by the prover and passed in as private witnesses. A dishonest prover could lie about these values. The correct fix is to compute them inside the circuit — but division and square roots are expensive in ZK. For production, these should be verified by a trusted third-party data feed or computed in-circuit using fixed-point arithmetic.

**No authentication.**
The merchant identity is a UUID passed in a request header (`X-Merchant-Id`). For the hackathon this is a hardcoded demo ID. Production requires proper auth (OAuth, wallet signature, etc.).

**Soroban contract is not yet deployed.**
The on-chain verification path depends on `indextree/ultrahonk_soroban_contract` being deployed to Stellar testnet. Until that happens, all verification runs off-chain through the backend.

**Single circuit version.**
There is one circuit (`lending_v1`) with a fixed set of 8 predicates. Lenders with different criteria (e.g. a 12-month window instead of 6, or additional metrics) would need a new circuit compiled and deployed. Circuit versioning and registry are not yet implemented.

**No statement deletion.**
The spec calls for deleting raw statements after proof generation. Currently raw text is stored in the `statements` table indefinitely. The PDF bytes themselves are not stored, but the extracted text is.
