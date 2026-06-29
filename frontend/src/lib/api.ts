import { LendingPolicy, MetricsSummary, ProofPackage, Transaction } from "./types";

const API = "/api";

const MERCHANT_ID = "00000000-0000-0000-0000-000000000001";

function headers(extra?: Record<string, string>) {
  return {
    "x-merchant-id": MERCHANT_ID,
    ...extra,
  };
}

export async function uploadStatement(file: File, month: string): Promise<{ statement_id: string; status: string }> {
  const form = new FormData();
  form.append("file", file);
  const res = await fetch(`${API}/upload-statement?month=${month}`, {
    method: "POST",
    headers: headers(),
    body: form,
  });
  if (!res.ok) throw new Error((await res.json()).error ?? res.statusText);
  return res.json();
}

export async function getTransactions(params?: { category?: string; limit?: number }): Promise<Transaction[]> {
  const qs = new URLSearchParams({
    merchant_id: MERCHANT_ID,
    ...(params?.category ? { category: params.category } : {}),
    limit: String(params?.limit ?? 200),
  });
  const res = await fetch(`${API}/transactions?${qs}`, { headers: headers() });
  if (!res.ok) throw new Error((await res.json()).error ?? res.statusText);
  return res.json();
}

export async function computeMetrics(): Promise<MetricsSummary> {
  const res = await fetch(`${API}/metrics`, {
    method: "POST",
    headers: headers(),
  });
  if (!res.ok) throw new Error((await res.json()).error ?? res.statusText);
  return res.json();
}

export async function generateProof(
  metricsId: string,
  policy: LendingPolicy
): Promise<{ proof_id: string; predicates: Array<{ name: string; description: string; satisfied: boolean }>; package: ProofPackage }> {
  const res = await fetch(`${API}/generate-proof`, {
    method: "POST",
    headers: { ...headers(), "content-type": "application/json" },
    body: JSON.stringify({ metrics_id: metricsId, policy }),
  });
  if (!res.ok) throw new Error((await res.json()).error ?? res.statusText);
  return res.json();
}

export async function verifyProof(proofPackage: ProofPackage): Promise<{ verified: boolean }> {
  const res = await fetch(`${API}/verify-proof`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ proof_package: proofPackage }),
  });
  if (!res.ok) throw new Error((await res.json()).error ?? res.statusText);
  return res.json();
}

export async function evaluateLoan(
  proofPackage: ProofPackage,
  policy: LendingPolicy
): Promise<{ decision: string; reason: string; proof_verified: boolean; failed_predicates: string[] }> {
  const res = await fetch(`${API}/loan/evaluate`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ proof_package: proofPackage, policy }),
  });
  if (!res.ok) throw new Error((await res.json()).error ?? res.statusText);
  return res.json();
}
