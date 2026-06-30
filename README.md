# LedgerProof

> **Prove financial health. Not financial history.**

LedgerProof is a privacy-preserving SME lending protocol. Businesses prove they meet loan criteria using zero-knowledge proofs — without handing over a single bank statement, customer name, or balance figure.

Lenders configure underwriting policies on-chain. Borrowers generate cryptographic proofs off-chain. A Soroban smart contract on Stellar verifies the proof and issues the decision. No financial documents change hands.

---

## The Problem

Every time a small business applies for a loan it hands over everything: six months of bank statements, customer names, supplier payments, salary runs, cash reserves, and margins. Most of that information has nothing to do with creditworthiness.

A lender only needs answers to a few binary questions:

- Is monthly revenue above ₦X?
- Has cash flow been positive for N consecutive months?
- Is account balance maintained above ₦Y?
- Are there any missed loan repayments?

Yet the standard process forces full financial disclosure. Every lender that a merchant approaches gets the complete picture — sensitive commercial data flowing to institutions they barely know, submitted repeatedly for every application.

**LedgerProof answers the lender's questions without answering anything else.**

---

## How It Works

### The Complete Flow

```
Borrower uploads bank statement (XLSX)
            │
            ▼
  Statement Parser — extracts every transaction
  into a normalised schema (date, description,
  credit, debit, balance)
            │
            ▼
  Financial Metrics Engine — computes 8 metrics
  in integer kobo arithmetic (no floating point)
            │
            ▼
  Noir Circuit — private inputs = merchant metrics
                 public inputs  = lender thresholds
            │
            ▼
  nargo execute  →  witness (.gz)
  bb write_vk    →  verification key
  bb prove       →  UltraHonk proof (~14 KB)
  bb verify      →  cryptographic confirmation
            │
            ▼
  Proof Package delivered to lender
  (proof + vk + public inputs — zero financial data)
            │
            ▼
  Soroban smart contract on Stellar
  verifies proof on-chain
            │
       ┌────┴────┐
    Approved   Rejected
```

No statements. No transactions. No balances. The lender learns only whether a mathematical predicate over private data is true.

---

## Screenshots

### Borrower — Upload Statement & Compute Metrics

The borrower uploads their XLSX bank statement. LedgerProof extracts every transaction, classifies it, and computes the financial summary locally. The numbers never leave the borrower's account.

![Borrower statement upload and financial summary](frontend/assets/Screenshot%202026-06-30%20at%2001.53.27.png)

---

### Borrower — Browse Lenders

Published lending desks are listed with their ZK criteria visible — minimum revenue, minimum balance, maximum volatility, maximum customer concentration. The borrower sees what they are being measured against before they apply.

![Browse published lenders](frontend/assets/Screenshot%202026-06-30%20at%2001.53.39.png)

---

### Borrower — My Applications

Applications are tracked per lender with live status (Pending → Approved / Rejected). The borrower never knows whether their documents were read — because they were never submitted.

![Borrower application tracker](frontend/assets/Screenshot%202026-06-30%20at%2001.53.47.png)

---

### Lender — Configure ZK Lending Policy

The lender publishes an underwriting policy: minimum thresholds and maximum tolerances expressed as integers (kobo, basis points). The live Naira/percentage hint updates as the lender types. These values become the public inputs committed into the ZK circuit.

![Lender policy configuration](frontend/assets/Screenshot%202026-06-30%20at%2001.53.58.png)

---

### Lender — Generate Proof & Loan Decision

The lender clicks **Generate Proof**. The UI steps through the full proving pipeline in real time — witness compilation, VK generation, UltraHonk proving, and cryptographic verification. The result shows each predicate individually (PASS / FAIL), the proof hash, verification key hash, proof size, and all public inputs committed to the circuit. The lender sees no financial data whatsoever.

![ZK proof generation and loan approval](frontend/assets/Screenshot%202026-06-30%20at%2001.55.24.png)

---

## Zero-Knowledge Proofs — In Plain Terms

A zero-knowledge proof lets one party (the prover) convince another (the verifier) that a statement is true without revealing *why* it is true or *what the underlying values are*.

### The Circuit

The underwriting circuit lives at [`circuits/lending/src/main.nr`](circuits/lending/src/main.nr). It is written in [Noir](https://noir-lang.org), a Rust-like ZK circuit language developed by Aztec.

**Private inputs** — the borrower's actual financial data. Known only to the borrower, mathematically hidden from the lender:

```
monthly_revenue              = [33_236_400, 34_441_800, 77_805_436, ...]   // 6 months, kobo
monthly_expenses             = [43_336_400, 45_121_800, 94_980_436, ...]
monthly_balances             = [83_666, 83_666, 83_666, ...]
revenue_volatility_bps       = 11_484    // 114.84%
customer_concentration_bps   = 5_600     // 56%
debt_ratio_bps               = 0
has_missed_repayments        = 0
account_age_months           = 5
```

**Public inputs** — the lender's thresholds. Committed into the proof. Visible to everyone:

```
required_monthly_revenue           = 80_000_000    // ₦800k
required_avg_balance               = 50_000        // ₦500
required_positive_cash_flow_months = 0
max_revenue_volatility_bps         = 12_000        // 120%
max_customer_concentration_bps     = 6_000         // 60%
max_debt_ratio_bps                 = 9_000         // 90%
require_no_missed_repayments       = 0
required_account_age_months        = 1
```

The circuit runs 8 constraint checks:

| # | Constraint |
|---|---|
| 1 | `avg(monthly_revenue) >= required_monthly_revenue` |
| 2 | `avg(monthly_balances) >= required_avg_balance` |
| 3 | `count(revenue > expenses) >= required_positive_cash_flow_months` |
| 4 | `revenue_volatility_bps <= max_revenue_volatility_bps` |
| 5 | `customer_concentration_bps <= max_customer_concentration_bps` |
| 6 | `debt_ratio_bps <= max_debt_ratio_bps` |
| 7 | `if require_no_missed_repayments: has_missed_repayments == 0` |
| 8 | `account_age_months >= required_account_age_months` |

If all constraints hold, Barretenberg generates a valid **UltraHonk proof** (~14 KB). If any constraint fails, no valid proof can be produced — it is mathematically impossible to fake a passing proof.

### The Proving Stack

| Step | Tool | What It Does |
|---|---|---|
| Circuit language | Noir | Typed DSL for arithmetic constraint systems |
| Witness generation | `nargo execute` | Runs the circuit on real inputs, produces a witness |
| Verification key | `bb write_vk --scheme ultra_honk` | Derives the key used to verify proofs for this circuit |
| Proof generation | `bb prove --scheme ultra_honk` | Constructs the UltraHonk cryptographic proof |
| Off-chain verification | `bb verify --scheme ultra_honk` | Verifies locally (primary demo path) |
| On-chain verification | Soroban contract on Stellar | Verifies on-chain, records loan decision |

### What the Lender Sees

```
✓  Monthly revenue meets minimum threshold          PASS
✓  Average balance meets minimum threshold          PASS
✓  Sufficient months of positive cash flow          PASS
✓  Revenue volatility within acceptable range       PASS
✓  No single customer dominates revenue             PASS
✓  Debt payments within acceptable ratio            PASS
✓  Account has sufficient history                   PASS

Proof ID:               a41097d8-1109-4eef-83da-e19c392b5bfe
Circuit:                lending_v1
Proof hash (32 bytes):  00000000000000000000000000000000...042ab5d6d1986846cf
VK hash (16 bytes):     00000000000010000000000000000c...
Proof size:             14,592 bytes
Verification:           ✓ VALID — UltraHonk verified
```

No amounts. No customer names. No transaction descriptions. No account numbers.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                      Browser (Next.js 15)               │
│                                                         │
│  /signup       /login       /borrower       /lender     │
│  Role picker   JWT auth     3-tab dash      2-tab dash  │
└──────────────────────────┬──────────────────────────────┘
                           │ REST (JWT Bearer)
                           │ /api/* → proxy → :3001
                           ▼
┌─────────────────────────────────────────────────────────┐
│                  Backend (Rust / Axum)                  │
│                                                         │
│  routes/                                                │
│    auth.rs          register, login (bcrypt + JWT)      │
│    statements.rs    XLSX upload and parsing             │
│    transactions.rs  normalised transaction listing      │
│    metrics.rs       financial metrics compute + fetch   │
│    lenders_api.rs   lender profile CRUD + publish       │
│    applications.rs  apply, list, ZK proof trigger       │
│    proofs.rs        direct proof generate / verify      │
│                                                         │
│  services/                                              │
│    xlsx_parser.rs   calamine XLSX extraction            │
│    metrics.rs       pure Rust 14-metric engine          │
│    proof_gen.rs     nargo + bb subprocess orchestrator  │
│    loan_engine.rs   policy evaluation + decision        │
└──────────────────────────┬──────────────────────────────┘
                           │ SQLx
                           ▼
┌─────────────────────────────────────────────────────────┐
│                   PostgreSQL                            │
│                                                         │
│  users               id, username, role, password_hash  │
│  lender_profiles     policy JSONB, published bool       │
│  loan_applications   borrower→lender, metrics, proof    │
│  financial_metrics   14 computed metrics per merchant   │
│  statements          uploaded XLSX metadata             │
│  transactions        normalised rows                    │
│  proofs              proof_hex, vk_hex, predicates      │
└──────────────────────────┬──────────────────────────────┘
                           │ subprocess
                           ▼
┌─────────────────────────────────────────────────────────┐
│              Noir / Barretenberg Toolchain              │
│                                                         │
│  circuits/lending/src/main.nr   8-predicate circuit     │
│  nargo execute                  witness generation      │
│  bb write_vk                    verification key        │
│  bb prove                       UltraHonk proof         │
│  bb verify                      cryptographic check     │
└─────────────────────────────────────────────────────────┘
                           │ (roadmap)
                           ▼
┌─────────────────────────────────────────────────────────┐
│           Soroban Smart Contract (Stellar)              │
│                                                         │
│  UltraHonk verifier (indextree/ultrahonk_soroban)       │
│  On-chain proof verification + loan record              │
└─────────────────────────────────────────────────────────┘
```

---

## Tech Stack

| Layer | Technology | Why |
|---|---|---|
| Backend | Rust + Axum | Memory-safe, zero-cost abstractions; ideal for financial and cryptographic workloads |
| Database | PostgreSQL + SQLx | Typed async queries; JSONB for flexible policy storage |
| Auth | JWT (HS256) + bcrypt | Stateless tokens; bcrypt cost-12 for password hashing |
| Statement parsing | Rust + calamine | Native XLSX parsing; no OCR dependencies, no LLM costs |
| Financial engine | Pure Rust, integer arithmetic | All values in kobo (integer kobo = no floating-point rounding errors) |
| ZK circuit | Noir | Strongly-typed constraint system; compiles to UltraHonk-compatible witness |
| Proving backend | Barretenberg (UltraHonk) | Sub-second proof generation for circuits of this size |
| On-chain | Soroban (Stellar) | Low fees, fast finality, Rust-native contract environment |
| Frontend | Next.js 15, TypeScript, Tailwind | App Router, server-side proxy, strict typing end-to-end |

---

## Project Structure

```
LedgerProof/
├── backend/
│   └── src/
│       ├── main.rs                 AppState, Axum server setup
│       ├── config.rs               Env var loading
│       ├── error.rs                Unified error → HTTP mapping
│       ├── models/
│       │   ├── user.rs             User, AuthClaims, RegisterRequest
│       │   ├── lender.rs           LenderProfile, UpsertProfileRequest
│       │   ├── application.rs      LoanApplication, CreateApplicationRequest
│       │   ├── metrics.rs          FinancialMetrics, LendingPolicy
│       │   ├── proof.rs            ProofPackage, ProvenPredicate
│       │   └── transaction.rs      Transaction schema
│       ├── routes/
│       │   ├── mod.rs              AuthUser JWT extractor + route table
│       │   ├── auth.rs             POST /auth/register, /auth/login
│       │   ├── statements.rs       POST /upload-statement
│       │   ├── transactions.rs     GET /transactions
│       │   ├── metrics.rs          POST /metrics, GET /metrics/latest
│       │   ├── lenders_api.rs      GET /lenders, POST /lenders/me
│       │   ├── applications.rs     Full application + proof trigger flow
│       │   └── proofs.rs           POST /generate-proof, /verify-proof
│       ├── services/
│       │   ├── xlsx_parser.rs      calamine extraction + row normalisation
│       │   ├── metrics.rs          14-metric financial engine
│       │   ├── proof_gen.rs        Prover.toml builder + nargo/bb runner
│       │   └── loan_engine.rs      Policy evaluation + decision
│       └── db/
│           └── migrations/
│               ├── 001_init.sql    Core tables (statements, transactions, metrics, proofs)
│               └── 002_users_lenders.sql  users, lender_profiles, loan_applications
│
├── circuits/
│   └── lending/
│       └── src/main.nr             8-predicate Noir underwriting circuit + unit tests
│
├── contracts/
│   └── lending_verifier/           Soroban smart contract (Stellar on-chain verification)
│
├── frontend/
│   ├── src/
│   │   ├── app/
│   │   │   ├── page.tsx            Marketing landing page
│   │   │   ├── login/page.tsx      Username + password sign-in
│   │   │   ├── signup/page.tsx     Role picker (borrower / lender) + registration
│   │   │   ├── borrower/page.tsx   3-tab: Statement · Browse Lenders · Applications
│   │   │   └── lender/page.tsx     2-tab: My Profile · Applications + proof panel
│   │   ├── lib/
│   │   │   ├── api.ts              All API calls with auth headers
│   │   │   ├── auth.ts             JWT storage helpers (localStorage)
│   │   │   └── types.ts            Shared TypeScript types
│   │   └── components/ui/          Button, Card, Badge, Input, Label
│   └── assets/                     App screenshots
│
└── docker-compose.yml
```

---

## Running Locally

### Prerequisites

| Tool | Version | Install |
|---|---|---|
| Rust | 1.78+ | [rustup.rs](https://rustup.rs) |
| Node.js | 20+ | [nodejs.org](https://nodejs.org) |
| PostgreSQL | 14+ | via brew, apt, or Docker |
| Nargo | 1.0.0-beta.9 | `curl -L noirup.dev \| bash && noirup` |
| Barretenberg | matching | `curl -L bbup.dev \| bash && bbup` |

### 1. Clone and configure

```bash
git clone https://github.com/thewoodfish/LedgerProof.git
cd LedgerProof
cp .env.example .env
```

Edit `.env`:

```env
DATABASE_URL=postgresql://ledgerproof:ledgerproof@localhost:5432/ledgerproof
JWT_SECRET=change-this-in-production
CIRCUITS_DIR=./circuits/lending
PORT=3001
RUST_LOG=ledgerproof_backend=debug
```

### 2. Create the database

```bash
# If PostgreSQL is running locally
createuser -s ledgerproof
createdb -O ledgerproof ledgerproof
# Or with docker
docker compose up -d postgres
```

### 3. Start the backend

The backend runs migrations automatically on startup.

```bash
# From repo root
cargo build
set -a && source .env && set +a
./target/debug/server
```

`▶ LedgerProof backend listening on 0.0.0.0:3001`

### 4. Start the frontend

```bash
cd frontend
npm install
npm run dev
```

`▶ Next.js ready on http://localhost:3002`

### 5. Verify the circuit toolchain

```bash
cd circuits/lending
nargo test          # runs 3 built-in circuit unit tests
nargo check         # type-checks the circuit
```

---

## Demo Walkthrough

### As a Lender

1. Go to `http://localhost:3002` → **Become a lender**
2. Sign up with role **Lender**
3. On the **My Lending Profile** tab:
   - Enter a display name (e.g. *QuickFund Capital*)
   - Review the ZK criteria — kobo thresholds with live Naira hints
   - Click **Save & Publish**
4. Switch to the **Applications** tab and wait for borrowers

### As a Borrower

1. Open a new browser / incognito window → **Apply for a loan**
2. Sign up with role **Borrower**
3. On the **My Statement** tab:
   - Upload your XLSX bank statement
   - Click **Compute Metrics** — financial summary appears (not sent to any lender)
4. On the **Browse Lenders** tab:
   - See published lenders and their criteria
   - Click **Apply** — a pending application is created
5. On the **Applications** tab:
   - Track status in real time

### Back as the Lender

1. Go to the **Applications** tab
2. See an anonymised applicant card — `Applicant #a284c02d` (no name, no financials)
3. Click **Generate Proof**
4. Watch the 4-step pipeline animate:
   - Compiling witness (`nargo execute`)
   - Writing verification key (`bb write_vk`)
   - Generating UltraHonk proof (`bb prove`)
   - Verifying proof (`bb verify`)
5. See the full result panel:
   - Verdict: `✓ LOAN APPROVED` or `✗ LOAN REJECTED`
   - Every predicate: PASS / FAIL
   - Proof hash, VK hash, proof size, circuit ID
   - All public inputs committed to the circuit

---

## API Reference

All endpoints except `/auth/*` and `GET /lenders` require `Authorization: Bearer <token>`.

### Auth

```
POST /auth/register    { username, password, role, full_name? }  → { token, user }
POST /auth/login       { username, password }                    → { token, user }
```

### Statements & Transactions

```
POST /upload-statement          multipart/form-data file=<xlsx>  → { statement_id }
GET  /transactions              → Transaction[]
POST /metrics                   → MetricsSummary
GET  /metrics/latest            → MetricsSummary
```

### Lenders

```
GET  /lenders                   (public)  → LenderProfile[]
GET  /lenders/me                (lender)  → LenderProfile
POST /lenders/me                (lender)  { display_name, description, policy } → LenderProfile
POST /lenders/me/publish        (lender)  → { published: bool }
```

### Applications

```
POST /applications              (borrower) { lender_profile_id, metrics_id }   → { application_id }
GET  /applications/mine         (borrower) → LoanApplication[]
GET  /applications/lender       (lender)   → LoanApplication[]  (anonymised)
POST /applications/:id/verify   (lender)   → VerifyResult + proof metadata
```

### Direct Proof Endpoints

```
POST /generate-proof   { metrics_id, policy }       → ProofPackage
POST /verify-proof     { proof_package }             → { verified: bool }
```

---

## Environment Variables

| Variable | Required | Description |
|---|---|---|
| `DATABASE_URL` | ✓ | PostgreSQL connection string |
| `JWT_SECRET` | ✓ | Secret key for HS256 JWT signing |
| `CIRCUITS_DIR` | ✓ | Absolute or relative path to `circuits/lending/` |
| `PORT` | | Backend port (default: `3001`) |
| `RUST_LOG` | | Logging filter (e.g. `ledgerproof_backend=debug`) |

---

## Security Properties

**Cryptographic guarantees**
- A valid UltraHonk proof cannot be generated for inputs that do not satisfy all circuit constraints. The lender cannot be deceived by a forged proof — the math makes it impossible.
- Public inputs (lender thresholds) are committed into the proof. A proof cannot be reused for a different set of thresholds.
- The verification key is derived deterministically from the circuit. Swapping the circuit invalidates the key.

**Data minimisation**
- XLSX statements are parsed in memory; raw bytes are not persisted after transaction extraction.
- `Prover.toml` (the file containing private financial inputs) is written to disk only for the duration of `nargo execute` and deleted immediately after.
- Lenders receive only: proof bytes, verification key, public inputs (thresholds), and predicate verdicts. No raw financial figures.
- Borrower identity is anonymised in the lender view — displayed as `Applicant #<first-8-of-UUID>`.

**Authentication**
- Passwords are hashed with bcrypt cost-12.
- JWTs expire after 7 days and are signed with HS256 using a server-side secret.
- All protected endpoints verify the token and extract the user's `id` and `role` before executing.

**Proof lock**
- A `Mutex` on `AppState` serialises concurrent proof generation requests. Barretenberg writes output to fixed paths inside the circuit directory; concurrent runs would corrupt each other's files.

### Known Limitations

| Limitation | Notes |
|---|---|
| Some metrics are pre-computed off-chain | Revenue volatility, customer concentration, and debt ratio are computed by the Rust engine and passed as private witnesses. A dishonest prover could misrepresent them. Production hardening requires computing them inside the circuit or attesting them from a trusted oracle. |
| Single concurrent proof | The proof lock serialises all proof requests. At scale, a dedicated proving cluster would generate proofs in parallel. |
| Off-chain verification is the primary demo path | On-chain Soroban verification requires deploying the UltraHonk verifier contract to Stellar testnet. The `bb verify` path is fully functional for demo purposes. |
| XLSX format assumption | The parser expects a specific column layout from Nigerian bank XLSX exports. Additional parsers per bank format are a Phase 2 item. |

---

## Roadmap

| Phase | Feature |
|---|---|
| ✅ MVP | XLSX upload · 8-metric ZK circuit · JWT auth · lender/borrower flow · UltraHonk proof UI |
| Phase 2 | Open Banking API — direct bank feed, no file upload required |
| Phase 3 | Accounting connectors — QuickBooks, Xero, Sage |
| Phase 4 | POS integrations — Paystack, Flutterwave, Moniepoint |
| Phase 5 | Inventory and supply chain proofs |
| Phase 6 | Tax compliance proofs |
| Phase 7 | Cross-bank reusable financial identity — one proof accepted by every lender |

---

## Why Stellar / Soroban

Stellar's sub-second finality and near-zero transaction fees make it practical to record proof verifications on-chain without the gas overhead of EVM chains. Soroban's Rust-native contract environment aligns directly with the backend stack and with Barretenberg's Rust bindings.

LedgerProof integrates with:
- [`indextree/ultrahonk_soroban_contract`](https://github.com/indextree/ultrahonk_soroban_contract) — production UltraHonk verifier for Soroban
- [`yugocabrio/rs-soroban-ultrahonk`](https://github.com/yugocabrio/rs-soroban-ultrahonk) — Rust integration layer for Soroban + Barretenberg

The Soroban contract receives the proof, public inputs, and circuit identifier. If verification succeeds, the contract records the loan decision immutably on-chain and triggers disbursement logic.

---

## Why Zero-Knowledge Proofs for Lending

Traditional lending creates a privacy dilemma: the lender needs enough information to assess risk, but collecting that information exposes the borrower's most sensitive commercial data. The standard resolution — share everything, assess manually — does not scale and creates significant data liability for lenders.

ZK proofs dissolve the dilemma. The borrower computes a proof that their metrics satisfy a threshold. The lender verifies the proof. The mathematics guarantee that a valid proof could not have been produced without satisfying every condition — so the lender can trust the result without seeing the data that produced it.

For SMEs in emerging markets, where access to formal finance is already constrained, this changes the calculus entirely:

- A single proof can be shared with multiple lenders — no repeated disclosure
- Lenders can automate underwriting against verifiable signals — no document review
- The borrower retains full control over their financial data — it never leaves their system

**This is the infrastructure layer that privacy-preserving SME finance needs.**

---

## License

MIT
