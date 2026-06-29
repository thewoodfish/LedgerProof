"use client";

import { useState, useCallback } from "react";
import Link from "next/link";
import { Shield, Upload, ChevronRight, RefreshCw, CheckCircle2, XCircle, AlertCircle, Copy, ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Progress } from "@/components/ui/progress";
import { computeMetrics, generateProof, getTransactions, uploadStatement } from "@/lib/api";
import { formatNaira, formatPct } from "@/lib/utils";
import type { MetricsSummary, ProofPackage, Transaction } from "@/lib/types";

type Step = "upload" | "transactions" | "metrics" | "proof";

const DEFAULT_POLICY = {
  required_monthly_revenue: 500_000_000,
  required_avg_balance: 50_000_000,
  required_positive_cash_flow_months: 4,
  max_revenue_volatility_bps: 1500,
  max_customer_concentration_bps: 2500,
  max_debt_ratio_bps: 2500,
  require_no_missed_repayments: true,
  required_account_age_months: 12,
};

const MONTHS = [
  "2024-01", "2024-02", "2024-03", "2024-04", "2024-05", "2024-06",
];

export default function MerchantPage() {
  const [step, setStep] = useState<Step>("upload");
  const [uploads, setUploads] = useState<{ month: string; file: File; status: string; id?: string }[]>([]);
  const [transactions, setTransactions] = useState<Transaction[]>([]);
  const [metrics, setMetrics] = useState<MetricsSummary | null>(null);
  const [proof, setProof] = useState<ProofPackage | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  // ── Upload phase ─────────────────────────────────────────────────────────
  const handleFileChange = (month: string, file: File | null) => {
    if (!file) return;
    setUploads((prev) => {
      const filtered = prev.filter((u) => u.month !== month);
      return [...filtered, { month, file, status: "pending" }];
    });
  };

  const handleUploadAll = async () => {
    if (uploads.length === 0) {
      setError("Please select at least one statement.");
      return;
    }
    setLoading(true);
    setError(null);
    const updated = [...uploads];
    for (let i = 0; i < updated.length; i++) {
      try {
        const res = await uploadStatement(updated[i].file, updated[i].month);
        updated[i] = { ...updated[i], status: "uploaded", id: res.statement_id };
      } catch (e: any) {
        updated[i] = { ...updated[i], status: "error" };
      }
    }
    setUploads(updated);
    setLoading(false);
    // Wait for parsing (background job) then move on
    setTimeout(() => setStep("transactions"), 2000);
  };

  // ── Transactions phase ────────────────────────────────────────────────────
  const handleLoadTransactions = async () => {
    setLoading(true);
    setError(null);
    try {
      const txns = await getTransactions({ limit: 300 });
      setTransactions(txns);
    } catch (e: any) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  // ── Metrics phase ─────────────────────────────────────────────────────────
  const handleComputeMetrics = async () => {
    setLoading(true);
    setError(null);
    try {
      const m = await computeMetrics();
      setMetrics(m);
      setStep("metrics");
    } catch (e: any) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  // ── Proof phase ───────────────────────────────────────────────────────────
  const handleGenerateProof = async () => {
    if (!metrics) return;
    setLoading(true);
    setError(null);
    try {
      const res = await generateProof(metrics.metrics_id, DEFAULT_POLICY);
      setProof(res.package);
      setStep("proof");
    } catch (e: any) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  };

  const handleCopyPackage = () => {
    if (!proof) return;
    navigator.clipboard.writeText(JSON.stringify(proof, null, 2));
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  // ── Render ────────────────────────────────────────────────────────────────
  const stepIndex = ["upload", "transactions", "metrics", "proof"].indexOf(step);
  const pct = ((stepIndex + 1) / 4) * 100;

  return (
    <div className="min-h-screen bg-slate-50">
      {/* Top bar */}
      <header className="bg-white border-b sticky top-0 z-10">
        <div className="max-w-4xl mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Link href="/" className="text-slate-400 hover:text-slate-700">
              <ArrowLeft className="h-4 w-4" />
            </Link>
            <Shield className="h-5 w-5 text-blue-600" />
            <span className="font-semibold">Merchant Portal</span>
          </div>
          <Progress value={pct} className="w-48 hidden sm:block" />
        </div>
      </header>

      <div className="max-w-4xl mx-auto px-6 py-10 space-y-8">
        {/* Step indicator */}
        <div className="flex items-center gap-2 text-sm">
          {["Upload", "Transactions", "Metrics", "Proof"].map((label, i) => (
            <div key={label} className="flex items-center gap-2">
              <div className={`w-6 h-6 rounded-full flex items-center justify-center text-xs font-bold
                ${i <= stepIndex ? "bg-blue-600 text-white" : "bg-slate-200 text-slate-500"}`}>
                {i + 1}
              </div>
              <span className={i <= stepIndex ? "text-slate-800 font-medium" : "text-slate-400"}>{label}</span>
              {i < 3 && <ChevronRight className="h-3 w-3 text-slate-300" />}
            </div>
          ))}
        </div>

        {error && (
          <div className="flex items-center gap-2 bg-red-50 border border-red-200 text-red-700 rounded-lg p-4 text-sm">
            <AlertCircle className="h-4 w-4 shrink-0" />
            {error}
          </div>
        )}

        {/* ── Step 1: Upload ─────────────────────────────────────────── */}
        {step === "upload" && (
          <Card>
            <CardHeader>
              <CardTitle>Upload Bank Statements</CardTitle>
              <CardDescription>
                Upload up to 6 months of statements. They're parsed locally — only ZK proofs leave your device.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid sm:grid-cols-2 gap-3">
                {MONTHS.map((month) => {
                  const upload = uploads.find((u) => u.month === month);
                  return (
                    <label
                      key={month}
                      className={`border-2 border-dashed rounded-lg p-4 cursor-pointer transition-colors
                        ${upload ? "border-blue-400 bg-blue-50" : "border-slate-200 hover:border-blue-300"}`}
                    >
                      <div className="flex items-center justify-between">
                        <div>
                          <p className="font-medium text-sm">{month}</p>
                          {upload ? (
                            <p className="text-xs text-slate-500 truncate max-w-[140px]">{upload.file.name}</p>
                          ) : (
                            <p className="text-xs text-slate-400">Click to select PDF</p>
                          )}
                        </div>
                        {upload?.status === "uploaded" ? (
                          <CheckCircle2 className="h-5 w-5 text-green-500" />
                        ) : upload?.status === "error" ? (
                          <XCircle className="h-5 w-5 text-red-500" />
                        ) : upload ? (
                          <Upload className="h-5 w-5 text-blue-500" />
                        ) : (
                          <Upload className="h-5 w-5 text-slate-300" />
                        )}
                      </div>
                      <input
                        type="file"
                        accept="application/pdf"
                        className="hidden"
                        onChange={(e) => handleFileChange(month, e.target.files?.[0] ?? null)}
                      />
                    </label>
                  );
                })}
              </div>

              <Button
                onClick={handleUploadAll}
                disabled={loading || uploads.length === 0}
                className="w-full gap-2"
              >
                {loading ? <RefreshCw className="h-4 w-4 animate-spin" /> : <Upload className="h-4 w-4" />}
                {loading ? "Uploading & Parsing..." : `Upload ${uploads.length} Statement${uploads.length !== 1 ? "s" : ""}`}
              </Button>

              {uploads.some((u) => u.status === "uploaded") && (
                <Button variant="outline" onClick={() => setStep("transactions")} className="w-full">
                  Continue to Transactions
                </Button>
              )}
            </CardContent>
          </Card>
        )}

        {/* ── Step 2: Transactions ───────────────────────────────────── */}
        {step === "transactions" && (
          <Card>
            <CardHeader>
              <CardTitle>Extracted Transactions</CardTitle>
              <CardDescription>
                Review the normalized transactions extracted from your statements.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              {transactions.length === 0 ? (
                <Button onClick={handleLoadTransactions} disabled={loading} className="w-full gap-2">
                  {loading ? <RefreshCw className="h-4 w-4 animate-spin" /> : null}
                  {loading ? "Loading..." : "Load Transactions"}
                </Button>
              ) : (
                <>
                  <div className="text-sm text-slate-500">
                    {transactions.length} transactions extracted across{" "}
                    {new Set(transactions.map((t) => t.date.slice(0, 7))).size} months
                  </div>
                  <div className="max-h-96 overflow-y-auto border rounded-lg divide-y">
                    {transactions.slice(0, 100).map((tx) => (
                      <div key={tx.id} className="px-4 py-3 flex items-center justify-between gap-4">
                        <div className="min-w-0">
                          <p className="text-sm font-medium truncate">{tx.description}</p>
                          <p className="text-xs text-slate-400">{tx.date}</p>
                        </div>
                        <div className="flex items-center gap-3 shrink-0">
                          {tx.credit > 0 && (
                            <span className="text-sm font-mono text-green-600">+{formatNaira(tx.credit)}</span>
                          )}
                          {tx.debit > 0 && (
                            <span className="text-sm font-mono text-red-500">-{formatNaira(tx.debit)}</span>
                          )}
                          <Badge variant={
                            tx.category === "revenue" ? "success" :
                            tx.category === "expense" ? "secondary" :
                            tx.category === "loan_repayment" ? "warning" : "outline"
                          } className="text-xs capitalize">
                            {tx.category.replace(/_/g, " ")}
                          </Badge>
                        </div>
                      </div>
                    ))}
                    {transactions.length > 100 && (
                      <div className="px-4 py-3 text-xs text-slate-400 text-center">
                        + {transactions.length - 100} more
                      </div>
                    )}
                  </div>
                  <Button onClick={handleComputeMetrics} disabled={loading} className="w-full gap-2">
                    {loading ? <RefreshCw className="h-4 w-4 animate-spin" /> : null}
                    {loading ? "Computing..." : "Compute Financial Metrics"}
                  </Button>
                </>
              )}
            </CardContent>
          </Card>
        )}

        {/* ── Step 3: Metrics ────────────────────────────────────────── */}
        {step === "metrics" && metrics && (
          <div className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>Financial Metrics</CardTitle>
                <CardDescription>
                  Computed from your transactions. These values stay private — only proofs are shared.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid sm:grid-cols-2 gap-4">
                  <MetricRow
                    label="Avg Monthly Revenue"
                    value={formatNaira(metrics.summary.avg_monthly_revenue_naira * 100)}
                    sub="kobo → naira display"
                    good={metrics.summary.avg_monthly_revenue_naira >= 5_000_000}
                  />
                  <MetricRow
                    label="Avg Monthly Balance"
                    value={formatNaira(metrics.summary.avg_monthly_balance_naira * 100)}
                    good={metrics.summary.avg_monthly_balance_naira >= 500_000}
                  />
                  <MetricRow
                    label="Positive CF Months"
                    value={`${metrics.summary.positive_cash_flow_months} / 6`}
                    good={metrics.summary.positive_cash_flow_months >= 4}
                  />
                  <MetricRow
                    label="Revenue Volatility"
                    value={formatPct(Math.round(metrics.summary.revenue_volatility_pct * 100))}
                    good={metrics.summary.revenue_volatility_pct <= 15}
                    invert
                  />
                  <MetricRow
                    label="Debt Ratio"
                    value={formatPct(Math.round(metrics.summary.debt_ratio_pct * 100))}
                    good={metrics.summary.debt_ratio_pct <= 25}
                    invert
                  />
                  <MetricRow
                    label="Customer Concentration"
                    value={formatPct(Math.round(metrics.summary.customer_concentration_pct * 100))}
                    good={metrics.summary.customer_concentration_pct <= 25}
                    invert
                  />
                  <MetricRow
                    label="Missed Repayments"
                    value={metrics.summary.has_missed_repayments ? "Yes" : "None"}
                    good={!metrics.summary.has_missed_repayments}
                  />
                  <MetricRow
                    label="Account Age"
                    value={`${metrics.summary.account_age_months} months`}
                    good={metrics.summary.account_age_months >= 12}
                  />
                </div>
              </CardContent>
            </Card>

            <Button onClick={handleGenerateProof} disabled={loading} size="lg" className="w-full gap-2">
              {loading ? <RefreshCw className="h-4 w-4 animate-spin" /> : <Shield className="h-4 w-4" />}
              {loading ? "Generating ZK Proof..." : "Generate Zero-Knowledge Proof"}
            </Button>
          </div>
        )}

        {/* ── Step 4: Proof ──────────────────────────────────────────── */}
        {step === "proof" && proof && (
          <div className="space-y-6">
            <Card className="border-green-200 bg-green-50">
              <CardHeader>
                <div className="flex items-center gap-2">
                  <CheckCircle2 className="h-6 w-6 text-green-600" />
                  <CardTitle className="text-green-800">ZK Proof Generated</CardTitle>
                </div>
                <CardDescription className="text-green-700">
                  Your proof package is ready. Share it with any lender — no financial data is included.
                </CardDescription>
              </CardHeader>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>Proven Predicates</CardTitle>
                <CardDescription>What the lender learns from your proof</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {proof.predicates.map((p) => (
                    <div key={p.name} className="flex items-center justify-between py-2 border-b last:border-0">
                      <div>
                        <p className="text-sm font-medium capitalize">{p.name}</p>
                        <p className="text-xs text-slate-400">{p.description}</p>
                      </div>
                      {p.satisfied ? (
                        <Badge variant="success" className="gap-1">
                          <CheckCircle2 className="h-3 w-3" /> Verified
                        </Badge>
                      ) : (
                        <Badge variant="destructive" className="gap-1">
                          <XCircle className="h-3 w-3" /> Failed
                        </Badge>
                      )}
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center justify-between">
                  Proof Package
                  <Button size="sm" variant="outline" onClick={handleCopyPackage} className="gap-1">
                    <Copy className="h-3 w-3" />
                    {copied ? "Copied!" : "Copy"}
                  </Button>
                </CardTitle>
                <CardDescription>
                  Share this JSON blob with lenders. Contains only the proof and public thresholds.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <pre className="bg-slate-950 text-slate-100 rounded-lg p-4 text-xs overflow-auto max-h-64">
                  {JSON.stringify(
                    {
                      proof_id: proof.proof_id,
                      circuit_id: proof.circuit_id,
                      public_inputs: proof.public_inputs,
                      predicates: proof.predicates,
                      proof_hex: proof.proof_hex.slice(0, 64) + "...",
                      vk_hex: proof.vk_hex.slice(0, 64) + "...",
                    },
                    null,
                    2
                  )}
                </pre>
              </CardContent>
            </Card>

            <div className="text-center">
              <Link href="/lender">
                <Button variant="outline" className="gap-2">
                  Open Lender Portal to verify this proof
                  <ChevronRight className="h-4 w-4" />
                </Button>
              </Link>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

function MetricRow({
  label,
  value,
  sub,
  good,
  invert = false,
}: {
  label: string;
  value: string;
  sub?: string;
  good: boolean;
  invert?: boolean;
}) {
  const ok = invert ? good : good;
  return (
    <div className="flex items-center justify-between p-3 rounded-lg bg-slate-50 border">
      <div>
        <p className="text-xs text-slate-500">{label}</p>
        <p className="font-semibold text-slate-800">{value}</p>
        {sub && <p className="text-xs text-slate-400">{sub}</p>}
      </div>
      {ok ? (
        <CheckCircle2 className="h-5 w-5 text-green-500 shrink-0" />
      ) : (
        <XCircle className="h-5 w-5 text-red-400 shrink-0" />
      )}
    </div>
  );
}
