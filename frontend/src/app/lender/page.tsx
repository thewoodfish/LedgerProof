"use client";

import { useState } from "react";
import Link from "next/link";
import { Shield, CheckCircle2, XCircle, AlertCircle, ArrowLeft, RefreshCw, Settings } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { evaluateLoan, verifyProof } from "@/lib/api";
import { formatNaira, formatPct } from "@/lib/utils";
import type { LendingPolicy, ProofPackage } from "@/lib/types";

const DEFAULT_POLICY: LendingPolicy = {
  required_monthly_revenue: 500_000_000,      // ₦5M in kobo
  required_avg_balance: 50_000_000,            // ₦500k in kobo
  required_positive_cash_flow_months: 4,
  max_revenue_volatility_bps: 1500,            // 15%
  max_customer_concentration_bps: 2500,        // 25%
  max_debt_ratio_bps: 2500,                    // 25%
  require_no_missed_repayments: true,
  required_account_age_months: 12,
};

type Stage = "input" | "verifying" | "result";

export default function LenderPage() {
  const [packageJson, setPackageJson] = useState("");
  const [policy, setPolicy] = useState<LendingPolicy>(DEFAULT_POLICY);
  const [stage, setStage] = useState<Stage>("input");
  const [result, setResult] = useState<{
    decision: string;
    reason: string;
    proof_verified: boolean;
    failed_predicates: string[];
  } | null>(null);
  const [verifyOnly, setVerifyOnly] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleEvaluate = async () => {
    setError(null);
    let pkg: ProofPackage;
    try {
      pkg = JSON.parse(packageJson);
    } catch {
      setError("Invalid JSON — paste the full proof package from the merchant portal.");
      return;
    }

    setStage("verifying");
    try {
      if (verifyOnly) {
        const r = await verifyProof(pkg);
        setResult({
          decision: r.verified ? "verified" : "invalid",
          reason: r.verified ? "Cryptographic proof is valid." : "Proof verification failed.",
          proof_verified: r.verified,
          failed_predicates: [],
        });
      } else {
        const r = await evaluateLoan(pkg, policy);
        setResult(r);
      }
      setStage("result");
    } catch (e: any) {
      setError(e.message);
      setStage("input");
    }
  };

  const handleReset = () => {
    setStage("input");
    setResult(null);
    setError(null);
  };

  return (
    <div className="min-h-screen bg-slate-50">
      <header className="bg-white border-b sticky top-0 z-10">
        <div className="max-w-4xl mx-auto px-6 py-4 flex items-center gap-3">
          <Link href="/" className="text-slate-400 hover:text-slate-700">
            <ArrowLeft className="h-4 w-4" />
          </Link>
          <Shield className="h-5 w-5 text-blue-600" />
          <span className="font-semibold">Lender Portal</span>
        </div>
      </header>

      <div className="max-w-4xl mx-auto px-6 py-10">
        <Tabs defaultValue="evaluate">
          <TabsList className="mb-8">
            <TabsTrigger value="evaluate">Evaluate Loan</TabsTrigger>
            <TabsTrigger value="policy">
              <Settings className="h-3.5 w-3.5 mr-1.5" />
              Configure Policy
            </TabsTrigger>
          </TabsList>

          {/* ── Evaluate tab ────────────────────────────────────────── */}
          <TabsContent value="evaluate" className="space-y-6">
            {error && (
              <div className="flex items-center gap-2 bg-red-50 border border-red-200 text-red-700 rounded-lg p-4 text-sm">
                <AlertCircle className="h-4 w-4 shrink-0" />
                {error}
              </div>
            )}

            {stage === "input" && (
              <Card>
                <CardHeader>
                  <CardTitle>Verify Proof Package</CardTitle>
                  <CardDescription>
                    Paste the JSON proof package received from the merchant. No financial data is contained — only cryptographic proofs.
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <textarea
                    className="w-full h-64 font-mono text-xs border rounded-lg p-4 bg-slate-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    placeholder='Paste proof package JSON here...\n{\n  "proof_id": "...",\n  "circuit_id": "lending_v1",\n  ...\n}'
                    value={packageJson}
                    onChange={(e) => setPackageJson(e.target.value)}
                  />

                  <div className="flex items-center gap-3 pt-1">
                    <label className="flex items-center gap-2 text-sm text-slate-600 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={verifyOnly}
                        onChange={(e) => setVerifyOnly(e.target.checked)}
                        className="rounded"
                      />
                      Verify proof only (skip policy check)
                    </label>
                  </div>

                  <Button
                    onClick={handleEvaluate}
                    disabled={!packageJson.trim()}
                    className="w-full gap-2"
                    size="lg"
                  >
                    <Shield className="h-4 w-4" />
                    {verifyOnly ? "Verify Proof" : "Evaluate & Decide"}
                  </Button>
                </CardContent>
              </Card>
            )}

            {stage === "verifying" && (
              <Card>
                <CardContent className="flex flex-col items-center justify-center py-16 gap-4">
                  <RefreshCw className="h-10 w-10 text-blue-600 animate-spin" />
                  <p className="font-medium text-slate-700">Verifying UltraHonk proof...</p>
                  <p className="text-sm text-slate-400">Running Barretenberg verifier</p>
                </CardContent>
              </Card>
            )}

            {stage === "result" && result && (
              <div className="space-y-6">
                {/* Decision banner */}
                <div className={`rounded-xl p-6 flex items-center gap-4 ${
                  result.decision === "approved"
                    ? "bg-green-50 border-2 border-green-200"
                    : "bg-red-50 border-2 border-red-200"
                }`}>
                  {result.decision === "approved" ? (
                    <CheckCircle2 className="h-10 w-10 text-green-600 shrink-0" />
                  ) : (
                    <XCircle className="h-10 w-10 text-red-500 shrink-0" />
                  )}
                  <div>
                    <h2 className={`text-2xl font-bold ${
                      result.decision === "approved" ? "text-green-800" : "text-red-800"
                    }`}>
                      Loan {result.decision === "approved" ? "APPROVED" : "REJECTED"}
                    </h2>
                    <p className={`text-sm mt-1 ${
                      result.decision === "approved" ? "text-green-700" : "text-red-700"
                    }`}>
                      {result.reason}
                    </p>
                  </div>
                </div>

                {/* Verification details */}
                <Card>
                  <CardHeader>
                    <CardTitle>Verification Details</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-3">
                    <div className="flex items-center justify-between py-2 border-b">
                      <span className="text-sm text-slate-500">Cryptographic proof</span>
                      {result.proof_verified ? (
                        <Badge variant="success" className="gap-1">
                          <CheckCircle2 className="h-3 w-3" /> Valid
                        </Badge>
                      ) : (
                        <Badge variant="destructive" className="gap-1">
                          <XCircle className="h-3 w-3" /> Invalid
                        </Badge>
                      )}
                    </div>
                    <div className="flex items-center justify-between py-2 border-b">
                      <span className="text-sm text-slate-500">Policy satisfied</span>
                      {result.failed_predicates.length === 0 ? (
                        <Badge variant="success" className="gap-1">
                          <CheckCircle2 className="h-3 w-3" /> Yes
                        </Badge>
                      ) : (
                        <Badge variant="destructive" className="gap-1">
                          <XCircle className="h-3 w-3" /> No ({result.failed_predicates.length} failed)
                        </Badge>
                      )}
                    </div>
                    {result.failed_predicates.length > 0 && (
                      <div className="pt-1">
                        <p className="text-xs text-slate-500 mb-2">Failed criteria:</p>
                        <div className="flex flex-wrap gap-2">
                          {result.failed_predicates.map((f) => (
                            <Badge key={f} variant="outline" className="text-red-600 border-red-200 text-xs">
                              {f}
                            </Badge>
                          ))}
                        </div>
                      </div>
                    )}
                  </CardContent>
                </Card>

                <div className="flex gap-3">
                  <Button onClick={handleReset} variant="outline" className="flex-1">
                    Evaluate Another
                  </Button>
                </div>
              </div>
            )}
          </TabsContent>

          {/* ── Policy tab ──────────────────────────────────────────── */}
          <TabsContent value="policy">
            <Card>
              <CardHeader>
                <CardTitle>Lending Policy</CardTitle>
                <CardDescription>
                  Configure your underwriting thresholds. These are matched against the ZK proof's proven predicates.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-5">
                <PolicyField
                  label="Minimum Monthly Revenue"
                  description="Average monthly revenue across 6 months"
                  value={formatNaira(policy.required_monthly_revenue ?? 0)}
                  rawValue={policy.required_monthly_revenue ?? 0}
                  unit="kobo"
                  onChange={(v) => setPolicy((p) => ({ ...p, required_monthly_revenue: v }))}
                  presets={[
                    { label: "₦3M", value: 300_000_000 },
                    { label: "₦5M", value: 500_000_000 },
                    { label: "₦10M", value: 1_000_000_000 },
                  ]}
                />
                <PolicyField
                  label="Minimum Average Balance"
                  description="Average closing balance across 6 months"
                  value={formatNaira(policy.required_avg_balance ?? 0)}
                  rawValue={policy.required_avg_balance ?? 0}
                  unit="kobo"
                  onChange={(v) => setPolicy((p) => ({ ...p, required_avg_balance: v }))}
                  presets={[
                    { label: "₦200k", value: 20_000_000 },
                    { label: "₦500k", value: 50_000_000 },
                    { label: "₦1M", value: 100_000_000 },
                  ]}
                />
                <PolicyField
                  label="Positive Cash Flow Months"
                  description="Minimum months with positive cash flow (out of 6)"
                  value={String(policy.required_positive_cash_flow_months ?? 0)}
                  rawValue={policy.required_positive_cash_flow_months ?? 0}
                  unit="months"
                  onChange={(v) => setPolicy((p) => ({ ...p, required_positive_cash_flow_months: v }))}
                  presets={[
                    { label: "3 mo", value: 3 },
                    { label: "4 mo", value: 4 },
                    { label: "6 mo", value: 6 },
                  ]}
                />
                <PolicyField
                  label="Max Revenue Volatility"
                  description="Coefficient of variation — lower is more stable"
                  value={formatPct(policy.max_revenue_volatility_bps ?? 0)}
                  rawValue={policy.max_revenue_volatility_bps ?? 0}
                  unit="bps"
                  onChange={(v) => setPolicy((p) => ({ ...p, max_revenue_volatility_bps: v }))}
                  presets={[
                    { label: "10%", value: 1000 },
                    { label: "15%", value: 1500 },
                    { label: "20%", value: 2000 },
                  ]}
                />
                <PolicyField
                  label="Max Customer Concentration"
                  description="Largest single customer as % of revenue"
                  value={formatPct(policy.max_customer_concentration_bps ?? 0)}
                  rawValue={policy.max_customer_concentration_bps ?? 0}
                  unit="bps"
                  onChange={(v) => setPolicy((p) => ({ ...p, max_customer_concentration_bps: v }))}
                  presets={[
                    { label: "20%", value: 2000 },
                    { label: "25%", value: 2500 },
                    { label: "30%", value: 3000 },
                  ]}
                />
                <PolicyField
                  label="Max Debt Ratio"
                  description="Loan repayments as % of revenue"
                  value={formatPct(policy.max_debt_ratio_bps ?? 0)}
                  rawValue={policy.max_debt_ratio_bps ?? 0}
                  unit="bps"
                  onChange={(v) => setPolicy((p) => ({ ...p, max_debt_ratio_bps: v }))}
                  presets={[
                    { label: "15%", value: 1500 },
                    { label: "25%", value: 2500 },
                    { label: "35%", value: 3500 },
                  ]}
                />

                <div className="flex items-center justify-between pt-2 border-t">
                  <div>
                    <p className="text-sm font-medium">Require No Missed Repayments</p>
                    <p className="text-xs text-slate-400">Reject if any loan payment gaps detected</p>
                  </div>
                  <button
                    onClick={() => setPolicy((p) => ({ ...p, require_no_missed_repayments: !p.require_no_missed_repayments }))}
                    className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                      policy.require_no_missed_repayments ? "bg-blue-600" : "bg-slate-300"
                    }`}
                  >
                    <span className={`inline-block h-4 w-4 transform rounded-full bg-white shadow transition-transform ${
                      policy.require_no_missed_repayments ? "translate-x-6" : "translate-x-1"
                    }`} />
                  </button>
                </div>

                <Button
                  className="w-full"
                  variant="outline"
                  onClick={() => setPolicy(DEFAULT_POLICY)}
                >
                  Reset to Default Policy
                </Button>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </div>
  );
}

function PolicyField({
  label,
  description,
  value,
  rawValue,
  unit,
  onChange,
  presets,
}: {
  label: string;
  description: string;
  value: string;
  rawValue: number;
  unit: string;
  onChange: (v: number) => void;
  presets: { label: string; value: number }[];
}) {
  return (
    <div className="space-y-2 pb-4 border-b last:border-0">
      <div className="flex items-center justify-between">
        <div>
          <p className="text-sm font-medium">{label}</p>
          <p className="text-xs text-slate-400">{description}</p>
        </div>
        <span className="text-sm font-mono font-bold text-slate-700">{value}</span>
      </div>
      <div className="flex gap-2">
        {presets.map((p) => (
          <button
            key={p.label}
            onClick={() => onChange(p.value)}
            className={`text-xs px-3 py-1 rounded-full border transition-colors ${
              rawValue === p.value
                ? "bg-blue-600 text-white border-blue-600"
                : "bg-white text-slate-600 border-slate-200 hover:border-blue-300"
            }`}
          >
            {p.label}
          </button>
        ))}
      </div>
    </div>
  );
}
