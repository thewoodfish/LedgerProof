import { getToken } from "./auth";
import type {
  AuthResponse,
  LendingPolicy,
  LenderProfile,
  LoanApplication,
  MetricsSummary,
  ProofPackage,
  Transaction,
  VerifyResult,
} from "./types";

const API = "/api";

function authHeaders(extra?: Record<string, string>): Record<string, string> {
  const token = getToken();
  return {
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
    ...extra,
  };
}

async function handleResponse<T>(res: Response): Promise<T> {
  if (!res.ok) {
    const body = await res.json().catch(() => ({ error: res.statusText }));
    throw new Error(body.error ?? res.statusText);
  }
  return res.json();
}

// ── Auth ───────────────────────────────────────────────────────────────────

export async function register(data: {
  username: string;
  password: string;
  role: string;
  full_name?: string;
}): Promise<AuthResponse> {
  const res = await fetch(`${API}/auth/register`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(data),
  });
  return handleResponse(res);
}

export async function login(data: {
  username: string;
  password: string;
}): Promise<AuthResponse> {
  const res = await fetch(`${API}/auth/login`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(data),
  });
  return handleResponse(res);
}

// ── Statements ─────────────────────────────────────────────────────────────

export async function uploadStatement(
  file: File
): Promise<{ statement_id: string; status: string }> {
  const form = new FormData();
  form.append("file", file);
  const res = await fetch(`${API}/upload-statement`, {
    method: "POST",
    headers: authHeaders(),
    body: form,
  });
  return handleResponse(res);
}

export async function getTransactions(params?: {
  category?: string;
  limit?: number;
}): Promise<Transaction[]> {
  const qs = new URLSearchParams({
    limit: String(params?.limit ?? 300),
    ...(params?.category ? { category: params.category } : {}),
  });
  const res = await fetch(`${API}/transactions?${qs}`, {
    headers: authHeaders(),
  });
  return handleResponse(res);
}

// ── Metrics ────────────────────────────────────────────────────────────────

export async function computeMetrics(): Promise<MetricsSummary> {
  const res = await fetch(`${API}/metrics`, {
    method: "POST",
    headers: authHeaders(),
  });
  return handleResponse(res);
}

export async function getLatestMetrics(): Promise<MetricsSummary> {
  const res = await fetch(`${API}/metrics/latest`, { headers: authHeaders() });
  return handleResponse(res);
}

// ── Lender profiles ────────────────────────────────────────────────────────

export async function getPublishedLenders(): Promise<LenderProfile[]> {
  const res = await fetch(`${API}/lenders`, { headers: authHeaders() });
  return handleResponse(res);
}

export async function getMyLenderProfile(): Promise<LenderProfile> {
  const res = await fetch(`${API}/lenders/me`, { headers: authHeaders() });
  return handleResponse(res);
}

export async function upsertLenderProfile(data: {
  display_name?: string;
  description?: string;
  policy?: LendingPolicy;
  published?: boolean;
  loan_amount_stroops?: number;
}): Promise<LenderProfile> {
  const res = await fetch(`${API}/lenders/me`, {
    method: "POST",
    headers: authHeaders({ "content-type": "application/json" }),
    body: JSON.stringify(data),
  });
  return handleResponse(res);
}

export async function togglePublish(): Promise<{ published: boolean }> {
  const res = await fetch(`${API}/lenders/me/publish`, {
    method: "POST",
    headers: authHeaders(),
  });
  return handleResponse(res);
}

// ── Applications ───────────────────────────────────────────────────────────

export async function createApplication(data: {
  lender_profile_id: string;
  metrics_id: string;
  amount_requested?: number;
}): Promise<{ application_id: string; status: string }> {
  const res = await fetch(`${API}/applications`, {
    method: "POST",
    headers: authHeaders({ "content-type": "application/json" }),
    body: JSON.stringify(data),
  });
  return handleResponse(res);
}

export async function getMyApplications(): Promise<LoanApplication[]> {
  const res = await fetch(`${API}/applications/mine`, { headers: authHeaders() });
  return handleResponse(res);
}

export async function getLenderApplications(): Promise<LoanApplication[]> {
  const res = await fetch(`${API}/applications/lender`, { headers: authHeaders() });
  return handleResponse(res);
}

export async function verifyApplication(id: string): Promise<VerifyResult> {
  const res = await fetch(`${API}/applications/${id}/verify`, {
    method: "POST",
    headers: authHeaders(),
  });
  return handleResponse(res);
}

// ── Proofs (kept for direct use) ───────────────────────────────────────────

export async function generateProof(
  metricsId: string,
  policy: LendingPolicy
): Promise<{ proof_id: string; predicates: ProvenPredicate[]; package: ProofPackage }> {
  const res = await fetch(`${API}/generate-proof`, {
    method: "POST",
    headers: authHeaders({ "content-type": "application/json" }),
    body: JSON.stringify({ metrics_id: metricsId, policy }),
  });
  return handleResponse(res);
}

export async function verifyProof(
  proofPackage: ProofPackage
): Promise<{ verified: boolean }> {
  const res = await fetch(`${API}/verify-proof`, {
    method: "POST",
    headers: authHeaders({ "content-type": "application/json" }),
    body: JSON.stringify({ proof_package: proofPackage }),
  });
  return handleResponse(res);
}

// ── User profile ───────────────────────────────────────────────────────────

export async function getMe(): Promise<{ id: string; username: string; role: string; stellar_address: string | null }> {
  const res = await fetch(`${API}/auth/me`, { headers: authHeaders() });
  return handleResponse(res);
}

export async function updateStellarAddress(address: string): Promise<{ stellar_address: string | null }> {
  const res = await fetch(`${API}/auth/stellar-address`, {
    method: "POST",
    headers: authHeaders({ "content-type": "application/json" }),
    body: JSON.stringify({ stellar_address: address }),
  });
  return handleResponse(res);
}

// ── Local type alias ───────────────────────────────────────────────────────
interface ProvenPredicate { name: string; description: string; satisfied: boolean; }
