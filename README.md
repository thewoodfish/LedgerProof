# LedgerProof

> **Prove financial health. Not financial history.**

LedgerProof is a privacy-preserving underwriting protocol that lets SMEs prove they meet lending criteria using zero-knowledge proofs — without handing over a single bank statement.

---

## The Problem

Every time a small business applies for a loan, it hands over everything: six months of bank statements, customer names, supplier payments, salary runs, cash reserves, margins. Most of that information is irrelevant to whether the business can repay a loan. The lender only needs answers to a few questions:

- Is monthly revenue above ₦5 million?
- Has cash flow been positive for at least four months?
- Is there a minimum balance of ₦500k maintained?
- Has the business missed any loan repayments?

Yet the standard process forces the merchant to expose their entire financial picture to get those questions answered. Every lender they approach gets the same full disclosure. Sensitive commercial data — customer concentration, supplier relationships, pricing signals — flows freely to institutions the merchant barely knows.

LedgerProof solves this by separating **what a lender needs to know** from **everything else**.

---

## How It Works

### The Merchant Side

1. **Upload a bank statement PDF.** The merchant uploads their bank statement through the merchant dashboard. Nothing else is required.

2. **Transactions are extracted automatically.** The backend parses the PDF using text extraction (lopdf), falling back to GPT-4o vision for scanned documents. Every transaction is normalized into a universal format: `date`, `description`, `credit`, `debit`, `balance`.

3. **14 financial metrics are computed.** A pure-Rust financial engine classifies transactions (revenue, expense, loan repayment, transfer, etc.) and computes metrics including monthly revenue, cash flow, average balance, revenue volatility, customer concentration, debt ratio, and account age.

4. **A zero-knowledge proof is generated.** The metrics are fed into a Noir circuit as private inputs. The lender's thresholds are the public inputs. The circuit runs 8 constraint checks and, if they all pass, produces a cryptographic proof — a small blob of bytes.

5. **The merchant receives a proof package.** This JSON object contains the proof, the verification key, and the public inputs (the thresholds). It contains no financial data whatsoever.

6. **The proof package is shared with any lender.** The same package can be sent to multiple lenders. No re-uploading, no re-disclosure.

### The Lender Side

1. **Receive the proof package** from the merchant (email, API, dashboard paste).
2. **Paste it into the lender portal** or submit it to `POST /loan/evaluate`.
3. **The proof is verified cryptographically.** The backend runs Barretenberg's UltraHonk verifier against the proof and public inputs.
4. **The lender's policy is checked.** The proven thresholds are compared against the lender's configured requirements. If the proof covers a threshold at least as strict as what the lender requires, it passes.
5. **Loan approved or declined.** Automatically. No analyst needed. No financial data viewed.

The lender learns only: *"This merchant satisfies these conditions."* Nothing else.

---

## Zero-Knowledge Proofs — In Plain Terms

A zero-knowledge proof is a cryptographic technique that lets one party (the prover) convince another party (the verifier) that a statement is true, without revealing *why* it is true.

### The Classic Analogy

Imagine you want to prove you know the solution to a maze, without showing anyone the path you took. A zero-knowledge proof lets you do exactly that — demonstrate knowledge without disclosure.

### How LedgerProof Uses It

The circuit (`circuits/lending/src/main.nr`) is a mathematical program written in [Noir](https://noir-lang.org/). It takes two kinds of inputs:

**Private inputs** — the merchant's actual financial data. These are known only to the merchant and never leave their control:
```
monthly_revenue              = [6_200_000_00, 5_800_000_00, ...]   // kobo
monthly_expenses             = [3_100_000_00, 2_900_000_00, ...]
monthly_balances             = [1_200_000_00,   980_000_00, ...]
revenue_volatility_bps       = 1100    // 11%
customer_concentration_bps   = 1800    // 18%
debt_ratio_bps               = 1200    // 12%
has_missed_repayments        = 0
account_age_months           = 28
```

**Public inputs** — the lender's thresholds. These are visible to everyone:
```
required_monthly_revenue           = 500_000_000   // ₦5M
required_avg_balance               = 50_000_000    // ₦500k
required_positive_cash_flow_months = 4
max_revenue_volatility_bps         = 1500          // 15%
max_customer_concentration_bps     = 2500          // 25%
max_debt_ratio_bps                 = 2500          // 25%
require_no_missed_repayments       = 1
required_account_age_months        = 12
```

The circuit runs 8 assertions:

| # | Check |
|---|---|
| 1 | Average monthly revenue ≥ required minimum |
| 2 | Average balance ≥ required minimum |
| 3 | Number of positive cash flow months ≥ required |
| 4 | Revenue volatility ≤ maximum allowed |
| 5 | Customer concentration ≤ maximum allowed |
| 6 | Debt ratio ≤ maximum allowed |
| 7 | No missed repayments (if lender requires it) |
| 8 | Account age ≥ required minimum |

If all 8 assertions hold, the Barretenberg prover generates a valid **UltraHonk proof**. If any assertion fails, no valid proof can be produced — it is mathematically impossible to fake a passing proof.

The lender runs the verifier against the proof and the public inputs. The verifier returns `true` or `false`. It learns nothing about the underlying financial values — only that they satisfy the stated conditions.

### What the Lender Sees

```
✓  Monthly revenue ≥ ₦5,000,000
✓  Average balance ≥ ₦500,000
✓  Positive cash flow for ≥ 4 months
✓  Revenue volatility ≤ 15%
✓  No single customer > 25% of revenue
✓  Debt payments ≤ 25% of revenue
✓  No missed loan repayments
✓  Account active ≥ 12 months
```

No amounts. No customer names. No transaction history.

### The Proving Stack

| Step | Tool | What it does |
|---|---|---|
| Circuit language | Noir | Defines the constraints in a typed DSL |
| Witness generation | `nargo execute` | Runs the circuit with actual inputs to produce a witness |
| Proof generation | `bb prove --scheme ultra_honk` | Generates the cryptographic proof |
| Verification key | `bb write_vk` | Writes the key used to verify the proof |
| Off-chain verification | `bb verify --scheme ultra_honk` | Verifies the proof locally (primary demo path) |
| On-chain verification | Soroban contract | Verifies the proof on Stellar and records the loan decision |

---

## Architecture

```
Merchant uploads PDF
        │
        ▼
Statement Parser (lopdf + GPT-4o)
        │
        ▼
Transaction Classifier (keyword rules, Rust)
        │
        ▼
Financial Metrics Engine (pure Rust, 14 metrics)
        │
        ▼
Prover.toml written (private inputs — deleted immediately after)
        │
        ▼
nargo execute → witness
        │
        ▼
bb prove → UltraHonk proof
        │
        ▼
Proof Package (proof + vk + public inputs — no financial data)
        │
        ▼
Lender verifies
        │
   ┌────┴────┐
  Off-chain  On-chain
  bb verify  Soroban contract
        │
        ▼
   Approved / Rejected
```

---

## Tech Stack

| Layer | Technology |
|---|---|
| Backend | Rust, Axum, SQLx, PostgreSQL |
| Statement parsing | lopdf + GPT-4o (vision fallback) |
| Financial engine | Pure Rust, integer arithmetic (kobo) |
| ZK circuit | Noir |
| Proving backend | Barretenberg (UltraHonk) |
| On-chain verification | Soroban smart contract (Stellar) |
| Frontend | Next.js 15, TypeScript, Tailwind CSS |
| Containerisation | Docker, docker-compose |

---

## Running Locally

### Prerequisites

- Rust 1.78+
- Node.js 20+
- Docker
- [Nargo](https://noir-lang.org/docs/getting_started/installation/) — `nargo` must be in PATH
- [Barretenberg](https://github.com/AztecProtocol/aztec-packages) — `bb` must be in PATH
- An OpenAI API key (for PDF parsing)

### Setup

```bash
# Clone the repo
git clone https://github.com/thewoodfish/LedgerProof.git
cd LedgerProof

# Copy env file and fill in your OpenAI key
cp .env.example .env

# Start PostgreSQL
docker compose up -d postgres

# Start the backend (from repo root)
cd backend
cargo run

# In another terminal, start the frontend
cd frontend
npm install
npm run dev
```

Visit `http://localhost:3000`.

### Environment Variables

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL connection string |
| `OPENAI_API_KEY` | Used for PDF statement parsing |
| `CIRCUITS_DIR` | Path to the Noir circuit directory (default: `../circuits/lending`) |
| `PORT` | Backend port (default: `3001`) |

---

## API

| Method | Endpoint | Description |
|---|---|---|
| `POST` | `/upload-statement` | Upload a bank statement PDF |
| `POST` | `/parse/{id}` | Re-trigger parsing for a statement |
| `GET` | `/transactions` | List extracted transactions |
| `POST` | `/metrics` | Compute financial metrics from transactions |
| `POST` | `/generate-proof` | Generate a UltraHonk ZK proof |
| `POST` | `/verify-proof` | Verify a proof package |
| `POST` | `/loan/evaluate` | Verify proof + evaluate lender policy |

---

## Project Structure

```
LedgerProof/
├── backend/                  Rust/Axum API server
│   └── src/
│       ├── routes/           HTTP handlers
│       ├── services/         Parser, metrics engine, proof generation
│       └── models/           Database types
├── circuits/
│   └── lending/              Noir circuit (lending_v1)
│       └── src/main.nr       8-predicate underwriting circuit
├── contracts/
│   └── lending_verifier/     Soroban smart contract (Stellar)
├── frontend/                 Next.js merchant & lender dashboards
└── docker-compose.yml
```

---

## Security Properties

- **Raw PDF bytes are never stored.** Only the extracted transaction text is retained.
- **Private inputs are written to disk only during proof generation** (`Prover.toml`) and deleted immediately after.
- **No bank credentials are ever collected.** The merchant uploads a PDF export — no login, no OAuth to their bank.
- **Proofs are cryptographically binding.** A valid UltraHonk proof cannot be constructed for inputs that do not satisfy the circuit constraints. The lender cannot be deceived by a forged proof.

### Known Limitations

- **Proof generation takes ~30 seconds.** Running `nargo` + `bb` serially is the bottleneck. Acceptable for demo; needs a dedicated proving service at production scale.
- **Some metrics are pre-computed off-chain.** Revenue volatility, customer concentration, and debt ratio are computed by the Rust backend and passed in as private witnesses. A dishonest prover could theoretically misrepresent these values. Production hardening requires computing them inside the circuit using fixed-point arithmetic, or attestation from a trusted data source.
- **Soroban on-chain verification requires the UltraHonk verifier to be deployed to Stellar testnet.** The off-chain path (`bb verify`) is the primary demo flow.
- **No production authentication.** Merchant identity is a UUID in a request header. Production requires wallet signatures or OAuth.

---

## Roadmap

| Phase | Feature |
|---|---|
| MVP | PDF upload, metric extraction, ZK proof generation, off-chain verification |
| Phase 2 | Open Banking API integration (direct bank feed, no PDF required) |
| Phase 3 | Accounting software connectors (QuickBooks, Xero, Sage) |
| Phase 4 | POS integrations (Paystack, Flutterwave, Moniepoint) |
| Phase 5 | Inventory and supply chain proofs |
| Phase 6 | Tax compliance proofs |
| Phase 7 | Cross-bank reusable financial identity |

---

## Why Stellar / Soroban

Stellar's low transaction costs and fast finality make it well-suited for recording proof verifications and loan decisions on-chain without the gas overhead of an EVM chain. Soroban's Rust-native smart contract environment also aligns directly with the backend stack. The existing [`indextree/ultrahonk_soroban_contract`](https://github.com/indextree/ultrahonk_soroban_contract) provides a production-ready UltraHonk verifier that LedgerProof plugs into directly via cross-contract call.

---

## License

MIT
