# LedgerProof

## Privacy-Preserving Merchant Financing Using Zero-Knowledge Proofs

### Technical Specification (MVP)

Version 1.0

---

# Vision

LedgerProof is a privacy-preserving underwriting protocol that enables businesses to prove financial health without revealing financial records.

Instead of uploading sensitive bank statements to every lender, merchants upload statements once, generate cryptographic proofs of selected financial metrics, and share only those proofs with lenders.

The lender learns only whether predefined lending requirements are satisfied.

Example:

Instead of revealing:

* Monthly Revenue = ₦6,482,313

The merchant proves:

> Monthly Revenue ≥ ₦5,000,000

without revealing the actual revenue.

LedgerProof is infrastructure that sits between merchant financial records and lending platforms.

---

# Problem

SMEs seeking financing today must disclose:

* Complete bank statements
* Customer names
* Suppliers
* Transaction history
* Pricing
* Margins
* Cash reserves
* Internal operations

Most of this information is irrelevant to underwriting.

Banks only need answers to questions like:

* Is revenue above X?
* Is cash flow positive?
* Has the merchant repaid previous loans?
* Is the business financially stable?

LedgerProof allows these questions to be answered using Zero-Knowledge Proofs instead of document sharing.

---

# Goals

### Merchant

* Keep financial data private
* Apply for loans faster
* Reuse proofs across lenders
* Eliminate repeated document submission

### Lender

* Faster underwriting
* Reduced fraud
* Automated loan approval
* Standardized financial verification

---

# High-Level Architecture

```
                    Merchant

                        │
        Upload Bank Statements (PDF)

                        │
                Statement Parser

                        │
             Normalized Transactions

                        │
          Financial Metrics Engine

                        │
            ZK Proof Generation

                        │
               Proof Package

                        │
               Lending Platform

                        │
            Automated Verification

                        │
             Loan Approved / Declined
```

---

# System Components

## 1. Statement Ingestion

Purpose

Receive merchant bank statements.

Supported formats

* PDF
* Image (future)
* CSV (future)
* Open Banking API (future)

Input

```
January.pdf
February.pdf
March.pdf
```

Output

Raw document.

---

## 2. Statement Parser

Purpose

Convert every statement into a universal transaction format.

Different banks use different layouts.

LedgerProof should normalize them into one schema.

Output

```json
{
  "date": "...",
  "description": "...",
  "credit": 25000,
  "debit": 0,
  "balance": 650000
}
```

The parser should extract:

* Date
* Description
* Debit
* Credit
* Balance

Parser Strategy

Phase 1

* OCR if necessary
* LLM-assisted extraction
* Validation rules

Future

Dedicated parsers per bank.

---

## 3. Transaction Classification

Purpose

Identify transaction types.

Categories

Revenue

Examples

* POS
* Card payment
* Transfer received

Expenses

Examples

* Supplier payment
* Utility bills
* Salary

Loan repayment

Examples

* Branch
* Carbon
* Moniepoint
* FairMoney

Transfers

Cash withdrawals

Taxes

Unknown

Classification can initially use an LLM before moving to deterministic rules.

---

# Financial Metrics Engine

This is the heart of LedgerProof.

Input

Normalized transactions.

Output

Financial indicators.

---

## Metric 1

Monthly Revenue

Formula

```
Sum(all incoming business payments)
```

Proof

```
Revenue >= Threshold
```

Example

```
Revenue >= ₦5,000,000
```

---

## Metric 2

Revenue Stability

Calculate

Coefficient of variation across N months.

Proof

```
Revenue volatility <= 15%
```

Purpose

Detect unstable businesses.

---

## Metric 3

Positive Cash Flow

Formula

```
Income - Expenses
```

Proof

```
Positive cash flow
for 6 consecutive months.
```

---

## Metric 4

Average Monthly Balance

Formula

Average closing balance.

Proof

```
Average balance >= Threshold
```

---

## Metric 5

Minimum Cash Reserve

Formula

Lowest account balance observed.

Proof

```
Minimum balance >= Threshold
```

---

## Metric 6

Revenue Growth

Formula

Month-over-month revenue growth.

Proof

```
Revenue increased
5 consecutive months.
```

---

## Metric 7

Business Activity

Formula

Number of incoming transactions.

Proof

```
Transactions >= 100
```

---

## Metric 8

Customer Diversity

Goal

Avoid dependence on one customer.

Compute

Largest customer contribution.

Proof

```
Largest customer <=25%
of revenue
```

---

## Metric 9

Supplier Diversity

Compute

Largest supplier contribution.

Proof

```
Largest supplier <=30%
of expenses
```

---

## Metric 10

Expense Stability

Measure

Variance in monthly expenses.

Proof

```
Expense variance <=20%
```

---

## Metric 11

Debt Ratio

Estimate

Recurring debt payments.

Proof

```
Debt payments <=25%
of revenue
```

---

## Metric 12

Loan Repayment History

Detect recurring loan payments.

Proof

```
No missed repayments.
```

---

## Metric 13

Account Age

Proof

```
Account active >=24 months.
```

---

## Metric 14

Transaction Frequency

Proof

```
>=300 transactions
per month.
```

---

# Zero-Knowledge Layer

Input

Financial metrics.

Output

Proof objects.

Example

Instead of

```
Revenue = ₦7.2M
```

Generate

```
Revenue >= ₦5M
```

Instead of

```
Balance = ₦1.8M
```

Generate

```
Balance >= ₦500k
```

No raw values leave the merchant.

---

# Lending Policy Engine

Lenders configure underwriting policies.

Example

```
Revenue >= ₦5M

Cash flow positive

Revenue volatility <=15%

Average balance >=₦300k

No missed repayments

Customer concentration <=25%
```

Each condition references a ZK proof.

If every proof verifies

Loan Approved

Else

Loan Declined

---

# Merchant Flow

Step 1

Upload six months of statements.

↓

Step 2

LedgerProof parses transactions.

↓

Step 3

Financial metrics calculated.

↓

Step 4

ZK proofs generated.

↓

Step 5

Merchant selects lender.

↓

Step 6

Proof package shared.

↓

Step 7

Loan decision returned.

---

# Lender Flow

Receive proof package.

↓

Verify cryptographic proofs.

↓

Evaluate lending policy.

↓

Approve or reject automatically.

No financial statements are ever viewed.

---

# Proof Package Example

```
Merchant

✓ Revenue >= ₦5M

✓ Positive cash flow

✓ Average balance >= ₦500k

✓ Revenue volatility <=15%

✓ Customer concentration <=25%

✓ No missed repayments
```

No additional financial information is disclosed.

---

# MVP Scope

## Frontend

Merchant Dashboard

* Upload statements
* View extracted transactions
* View computed metrics
* Generate proofs
* Share proof package

Lender Dashboard

* Receive proof package
* Verify proofs
* Configure lending rules
* Approve/reject loan

---

## Backend

Modules

Statement Service

Financial Analysis Service

Proof Generation Service

Verification Service

Loan Decision Engine

---

## Suggested Tech Stack

### Frontend

Next.js

TypeScript

Tailwind CSS

shadcn/ui

React Hook Form

---

### Backend

Rust (preferred)

Axum

SQLx

PostgreSQL

Redis (optional)

---

### Document Processing

Rust PDF parser

OCR (when required)

LLM extraction fallback

---

### Financial Engine

Pure Rust

Deterministic calculations

Unit-tested formulas

---

### Zero-Knowledge

## Zero-Knowledge Layer

LedgerProof uses **Noir** for circuit development and **Barretenberg's UltraHonk proving system** to generate efficient zero-knowledge proofs. The system is designed to produce succinct proofs for lending predicates (e.g., `monthly_revenue >= ₦5,000,000`) while keeping all underlying financial records private.

### ZK Stack

| Component             | Technology                 |
| --------------------- | -------------------------- |
| Circuit Language      | Noir                       |
| Proving Backend       | Barretenberg (UltraHonk)   |
| Proof Generation      | UltraHonk Prover           |
| On-chain Verification | Soroban UltraHonk Verifier |
| Smart Contracts       | Soroban (Rust)             |

### Soroban Integration

LedgerProof will leverage existing UltraHonk verification implementations for Soroban:

* **UltraHonk Soroban Verifier**

  * `indextree/ultrahonk_soroban_contract`

* **Rust Integration**

  * `yugocabrio/rs-soroban-ultrahonk`

These repositories provide the verifier implementation required to validate Noir-generated UltraHonk proofs directly within Soroban smart contracts.

---

## Proof Flow

```text
Merchant uploads bank statements
            │
            ▼
Statement Parser
            │
            ▼
Financial Metrics Engine
            │
            ▼
Noir Witness Generation
            │
            ▼
UltraHonk Proof Generation
            │
            ▼
Proof Package
            │
            ▼
Soroban Smart Contract
(UltraHonk Verifier)
            │
            ▼
Proof Valid?
      │            │
     Yes           No
      │            │
Approve Loan    Reject
```

---

## Circuit Design

Each lending criterion is represented as a separate Noir circuit (or composed into a larger underwriting circuit).

Example predicates include:

* `monthly_revenue >= threshold`
* `cash_flow_positive == true`
* `average_balance >= threshold`
* `debt_ratio <= threshold`
* `revenue_volatility <= threshold`
* `no_missed_repayments == true`
* `customer_concentration <= threshold`

The circuits consume **private financial metrics** and expose only **public boolean assertions** or threshold commitments.

Example:

Private Inputs

```text
monthly_revenue = ₦6,482,313
average_balance = ₦1,120,000
```

Public Inputs

```text
required_revenue = ₦5,000,000
required_balance = ₦500,000
```

Public Outputs

```text
RevenueRequirement = TRUE
BalanceRequirement = TRUE
```

The lender learns only that the conditions are satisfied—not the underlying financial values.

---

## Proof Package

A generated proof package contains:

* UltraHonk proof
* Public inputs
* Circuit identifier
* Proof metadata
* Merchant identifier (or anonymous application ID)
* Timestamp

Raw bank statements, transactions, and computed financial metrics are **never included**.

---

## Soroban Verification

The lending contract receives:

* Proof
* Public inputs
* Circuit identifier

Using the UltraHonk verifier deployed on Soroban, the contract performs on-chain verification.

If verification succeeds, the contract proceeds with automated underwriting according to the lender's policy.

Example:

```text
Verify UltraHonk Proof
        │
        ▼
Valid?
        │
 ┌──────┴──────┐
 │             │
Yes           No
 │             │
Continue     Reject
```

---

## Design Principles

* Financial records remain entirely off-chain.
* Only cryptographic proofs and public verification inputs are submitted to Soroban.
* Proof generation occurs off-chain on trusted merchant infrastructure or LedgerProof servers.
* Soroban is responsible only for proof verification and lending logic.
* The architecture is modular, allowing additional Noir circuits to be added as new underwriting metrics are introduced.

This approach keeps proof generation computationally efficient, minimizes on-chain costs, and leverages the existing Noir → Barretenberg → UltraHonk → Soroban ecosystem for end-to-end verifiable underwriting.


### Blockchain

Stellar

Responsibilities

* Store proof commitments or proof references
* Verify proof metadata (if applicable)
* Record loan issuance
* Record repayment history (future)

Sensitive financial data should remain off-chain.

---

# API Design

POST

```
/upload-statement
```

POST

```
/parse
```

GET

```
/transactions
```

POST

```
/metrics
```

POST

```
/generate-proof
```

POST

```
/verify-proof
```

POST

```
/loan/evaluate
```

---

# Security

Never store plaintext financial records longer than necessary.

Encrypt uploaded statements.

Delete statements after proof generation (configurable retention).

Store only:

* Proofs
* Metric commitments
* Verification metadata
* Audit logs

No bank credentials should ever be collected.

---

# Future Roadmap

### Phase 2

Open Banking integrations.

Automatic statement sync.

---

### Phase 3

Accounting software integrations.

* QuickBooks
* Xero
* Sage

---

### Phase 4

POS integrations.

* Square
* Moniepoint POS
* Paystack
* Flutterwave

---

### Phase 5

Inventory verification.

Generate proofs for:

* Inventory turnover
* Stock levels
* Sales velocity

---

### Phase 6

Tax compliance proofs.

---

### Phase 7

Cross-bank reputation.

One reusable financial identity accepted by multiple lenders.

---

# Success Criteria (Hackathon MVP)

A successful demo should show the complete flow:

1. Merchant uploads 6 months of bank statements.
2. LedgerProof extracts and normalizes transactions.
3. Financial metrics are computed automatically.
4. Zero-knowledge proofs are generated for selected lending criteria.
5. A Stellar-based lending application verifies those proofs.
6. The loan is automatically approved or declined without exposing any raw financial data.

---

# Long-Term Vision

LedgerProof becomes the privacy layer for SME finance.

Instead of sharing financial documents, businesses share cryptographic proofs of financial health. Lenders receive trustworthy underwriting signals, merchants retain control of their data, and financial access becomes faster, more secure, and privacy-preserving.

**Tagline**

> **Prove financial health. Not financial history.**
