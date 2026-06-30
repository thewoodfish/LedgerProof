# LedgerProof — Demo Script

**Target length:** 3 minutes max.

---

## OPENING (0:00 – 0:20)

*Show: landing page*

> "Every time a small business applies for a loan, they hand over everything — bank statements, customer names, transaction history, margins. Most of it is irrelevant. LedgerProof fixes that. Businesses prove they qualify using zero-knowledge proofs. Lenders get a cryptographic answer. No documents change hands."

---

## LENDER SETS POLICY (0:20 – 0:50)

*Show: sign up as lender → lender dashboard → My Lending Profile tab*

> "A lender signs up and configures their underwriting criteria — minimum revenue, minimum balance, maximum volatility, customer concentration limits. These thresholds become the public inputs committed into the ZK circuit."

*Click Save & Publish.*

> "They publish. Borrowers can now see this lender and apply."

---

## BORROWER UPLOADS STATEMENT (0:50 – 1:30)

*Show: open new tab → sign up as borrower → borrower dashboard → My Statement tab*

> "A borrower signs up and uploads their bank statement — just an XLSX export from their bank."

*Upload the file.*

> "LedgerProof parses every transaction and classifies it. Then we compute the financial metrics."

*Click Compute Metrics → financial summary appears.*

> "Average monthly revenue, average balance, revenue volatility, customer concentration, debt ratio, account age — all computed privately. These numbers never leave this dashboard."

---

## BORROWER APPLIES (1:30 – 1:45)

*Show: Browse Lenders tab → lender card with criteria chips*

> "The borrower can see all published lenders and the exact criteria they'll be measured against."

*Click Apply.*

> "One click to apply. No documents submitted. The lender receives nothing yet."

---

## LENDER GENERATES PROOF (1:45 – 2:30)

*Switch back to lender tab → Applications tab*

> "On the lender side, an anonymised application comes in. No name, no financials — just an applicant ID and a timestamp."

*Click Generate Proof.*

> "The lender triggers proof generation. Watch the pipeline."

*Let the 5 steps animate — narrate each:*

> "First, nargo compiles a witness from the borrower's financial metrics. Then Barretenberg derives the UltraHonk verification key. The prover constructs a 14-kilobyte cryptographic proof. bb verify confirms it's valid. And finally — the decision is recorded on Stellar."

*LOAN APPROVED banner appears.*

---

## SHOW THE RESULT (2:30 – 2:50)

*Show: predicate list, proof metadata, Stellar section*

> "Every predicate — PASS. The proof hash, verification key hash, proof size, circuit ID. And here — a live Stellar transaction. The loan decision is stored permanently on-chain. Anyone can verify it."

*Click "View on Stellar Expert" → show the transaction in the browser.*

> "That's a real transaction on Stellar testnet. Contract `CDY7T3...`. Immutable record."

---

## CLOSE (2:50 – 3:00)

*Show: landing page tagline*

> "LedgerProof. Prove financial health. Not financial history."

---

## Tips Before Recording

- Use two browser windows or two profiles — one logged in as lender, one as borrower
- Make sure the backend is running and testnet is reachable before you hit record
- The proof generation takes a few seconds — don't cut it, let the animation play
- Zoom in on the Stellar tx hash section — that's the money shot for the judges
