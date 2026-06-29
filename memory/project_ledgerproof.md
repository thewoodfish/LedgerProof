---
name: project-ledgerproof
description: LedgerProof — full system built, architecture decisions, run instructions
metadata:
  type: project
---

Full system built end-to-end. All layers compile and circuit tests pass.

**Why:** Hackathon MVP — privacy-preserving SME financing using ZK proofs on Stellar/Soroban.

**Stack:**
- Backend: Rust/Axum + SQLx + PostgreSQL (`backend/`)
- ZK: Noir circuit + Barretenberg UltraHonk (`circuits/lending/`)
- Smart contract: Soroban Rust (`contracts/lending_verifier/`)
- Frontend: Next.js 15 + Tailwind + shadcn-style UI (`frontend/`)

**To run:**
1. `docker compose up -d` (starts Postgres)
2. `cp .env.example .env` and fill in `ANTHROPIC_API_KEY`
3. `cargo run -p ledgerproof-backend --bin server`
4. `cd frontend && npm run dev`

**Proof CLI flags (bb 0.87.0):**
- prove: `bb prove --scheme ultra_honk -b target/lending.json -w target/lending.gz -o target/proof`
- vk: `bb write_vk --scheme ultra_honk -b target/lending.json -o target/vk`
- verify: `bb verify --scheme ultra_honk -k target/vk/vk -p target/proof/proof -i target/proof/public_inputs`

**How to apply:** The Prover.toml and proof generation are handled by `backend/src/services/proof_gen.rs`. The circuit is at `circuits/lending/src/main.nr`. Public inputs are amounts in kobo (100 kobo = NGN 1), metrics as basis points.
