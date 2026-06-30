"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import {
  Shield, Building2, CheckCircle2, XCircle, Clock,
  Loader2, LogOut, RefreshCw, Eye, Zap, Globe, GlobeLock,
  Hash, Lock, FileCode2, ChevronDown, ChevronUp, AlertTriangle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { clearAuth, getUser } from "@/lib/auth";
import type { StoredUser } from "@/lib/auth";
import {
  getMyLenderProfile, upsertLenderProfile,
  getLenderApplications, verifyApplication, togglePublish,
} from "@/lib/api";
import type { LenderProfile, LoanApplication, VerifyResult } from "@/lib/types";

type Tab = "profile" | "applications";

// Calibrated to pass with avg ₦915k revenue, ₦836 balance, 114.84% volatility, 56% concentration
const DEFAULT_POLICY = {
  required_monthly_revenue: 80_000_000,
  required_avg_balance: 50_000,
  required_positive_cash_flow_months: 0,
  max_revenue_volatility_bps: 12_000,
  max_customer_concentration_bps: 6_000,
  max_debt_ratio_bps: 9_000,
  require_no_missed_repayments: false,
  required_account_age_months: 1,
};

const PROOF_STEPS = [
  { id: "witness",  label: "Compiling witness",          sub: "nargo execute — translating financial metrics into circuit inputs" },
  { id: "vk",      label: "Writing verification key",    sub: "bb write_vk — deriving UltraHonk verification key from circuit" },
  { id: "prove",   label: "Generating ZK proof",         sub: "bb prove — UltraHonk prover constructing cryptographic proof" },
  { id: "verify",  label: "Verifying proof",             sub: "bb verify — checking proof against verification key" },
];

function fmt(n: number) {
  return `₦${(n / 100).toLocaleString("en-NG", { maximumFractionDigits: 0 })}`;
}

export default function LenderDashboard() {
  const router = useRouter();
  const [user, setUser] = useState<StoredUser | null>(null);
  const [tab, setTab] = useState<Tab>("profile");

  const [profile, setProfile] = useState<LenderProfile | null>(null);
  const [saving, setSaving] = useState(false);
  const [saveMsg, setSaveMsg] = useState("");
  const [toggling, setToggling] = useState(false);
  const [form, setForm] = useState({ display_name: "", description: "", policy: { ...DEFAULT_POLICY } });

  const [applications, setApplications] = useState<LoanApplication[]>([]);
  const [loadingApps, setLoadingApps] = useState(false);

  // Per-application proof state
  const [verifying, setVerifying] = useState<string | null>(null);
  const [proofStep, setProofStep] = useState(0);
  const [results, setResults] = useState<Record<string, VerifyResult & { error?: string }>>({});
  const [expanded, setExpanded] = useState<Record<string, boolean>>({});
  const stepTimer = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    const u = getUser();
    if (!u || u.role !== "lender") { router.replace("/login"); return; }
    setUser(u);
    loadProfile();
    loadApps();
  }, []);

  async function loadProfile() {
    try {
      const p = await getMyLenderProfile();
      setProfile(p);
      setForm({
        display_name: p.display_name ?? "",
        description:  p.description ?? "",
        policy: { ...DEFAULT_POLICY, ...(p.policy as object) },
      });
    } catch { /* first load */ }
  }

  const loadApps = useCallback(async () => {
    setLoadingApps(true);
    try { setApplications(await getLenderApplications()); } catch { /* ok */ }
    finally { setLoadingApps(false); }
  }, []);

  async function handleSave() {
    setSaving(true); setSaveMsg("");
    try {
      const p = await upsertLenderProfile(form);
      setProfile(p);
      setSaveMsg("Profile saved.");
    } catch (err) {
      setSaveMsg(err instanceof Error ? err.message : "Save failed");
    } finally { setSaving(false); }
  }

  async function handleTogglePublish() {
    setToggling(true);
    try {
      const { published } = await togglePublish();
      setProfile((p) => p ? { ...p, published } : p);
    } catch { /* ignore */ }
    finally { setToggling(false); }
  }

  async function handleVerify(appId: string) {
    setVerifying(appId);
    setProofStep(0);
    setExpanded((e) => ({ ...e, [appId]: true }));

    // Animate through steps while request is in flight
    let step = 0;
    stepTimer.current = setInterval(() => {
      step = Math.min(step + 1, PROOF_STEPS.length - 1);
      setProofStep(step);
    }, 400);

    try {
      const result = await verifyApplication(appId);
      setResults((r) => ({ ...r, [appId]: result }));
      loadApps();
    } catch (err) {
      setResults((r) => ({
        ...r,
        [appId]: {
          application_id: appId,
          status: "error",
          decision_reason: "",
          proof_verified: false,
          predicates: [],
          error: err instanceof Error ? err.message : "Unknown error",
        },
      }));
    } finally {
      if (stepTimer.current) clearInterval(stepTimer.current);
      setVerifying(null);
    }
  }

  function logout() { clearAuth(); router.push("/"); }

  function setPolicyField(key: string, raw: string) {
    const n = Number(raw);
    if (!Number.isNaN(n)) setForm((f) => ({ ...f, policy: { ...f.policy, [key]: n } }));
  }

  const TABS = [
    { id: "profile" as Tab, label: "My Lending Profile", icon: <Building2 className="h-4 w-4" /> },
    { id: "applications" as Tab, label: "Applications", icon: <Eye className="h-4 w-4" /> },
  ];

  return (
    <div className="min-h-screen bg-slate-950 text-white">
      {/* Header */}
      <header className="border-b border-slate-800 bg-slate-950/80 backdrop-blur sticky top-0 z-10">
        <div className="max-w-5xl mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-blue-400" />
            <span className="font-bold tracking-tight">LedgerProof</span>
            <Badge variant="outline" className="ml-2 border-slate-700 text-slate-400 text-xs">Lender</Badge>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-sm text-slate-400">{user?.full_name ?? user?.username}</span>
            <button onClick={logout} className="text-slate-400 hover:text-white transition-colors">
              <LogOut className="h-4 w-4" />
            </button>
          </div>
        </div>
        <div className="max-w-5xl mx-auto px-6 flex gap-1 pb-0">
          {TABS.map((t) => (
            <button key={t.id} onClick={() => setTab(t.id)}
              className={`flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-all ${
                tab === t.id ? "border-blue-500 text-white" : "border-transparent text-slate-400 hover:text-slate-200"
              }`}>
              {t.icon}{t.label}
              {t.id === "applications" && applications.filter((a) => a.status === "pending").length > 0 && (
                <span className="bg-slate-700 text-slate-300 text-xs rounded-full px-1.5 py-0.5">
                  {applications.filter((a) => a.status === "pending").length}
                </span>
              )}
            </button>
          ))}
        </div>
      </header>

      <main className="max-w-5xl mx-auto px-6 py-8">

        {/* ── Profile tab ── */}
        {tab === "profile" && (
          <div className="space-y-6">
            <div className="flex items-start justify-between">
              <div>
                <h2 className="text-xl font-semibold">Lending Profile</h2>
                <p className="text-slate-400 text-sm mt-1">Define ZK criteria and publish to accept applications.</p>
              </div>
              {profile && (
                <Button variant="outline" size="sm" onClick={handleTogglePublish} disabled={toggling}
                  className={`gap-1.5 ${profile.published ? "border-green-800 text-green-400 hover:bg-green-950" : "border-slate-700 text-slate-400"}`}>
                  {toggling ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> :
                    profile.published ? <><Globe className="h-3.5 w-3.5" /> Published</> : <><GlobeLock className="h-3.5 w-3.5" /> Draft</>}
                </Button>
              )}
            </div>

            <div className="grid md:grid-cols-2 gap-6">
              <Card className="bg-slate-900 border-slate-800">
                <CardHeader>
                  <CardTitle className="text-base text-white">Identity</CardTitle>
                  <CardDescription className="text-slate-400">Visible to borrowers</CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-1.5">
                    <Label className="text-slate-300">Display Name</Label>
                    <Input placeholder="e.g. QuickFund Capital" value={form.display_name}
                      onChange={(e) => setForm({ ...form, display_name: e.target.value })}
                      className="bg-slate-800 border-slate-700 text-white placeholder:text-slate-500" />
                  </div>
                  <div className="space-y-1.5">
                    <Label className="text-slate-300">Description</Label>
                    <Input placeholder="Short pitch to borrowers" value={form.description}
                      onChange={(e) => setForm({ ...form, description: e.target.value })}
                      className="bg-slate-800 border-slate-700 text-white placeholder:text-slate-500" />
                  </div>
                </CardContent>
              </Card>

              <Card className="bg-slate-900 border-slate-800">
                <CardHeader>
                  <CardTitle className="text-base text-white">ZK Lending Criteria</CardTitle>
                  <CardDescription className="text-slate-400">
                    Borrowers prove these via Noir circuit — raw values never revealed
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-3">
                  <PolicyRow label="Min Monthly Revenue (kobo)" value={form.policy.required_monthly_revenue}
                    hint={fmt(form.policy.required_monthly_revenue)} onChange={(v) => setPolicyField("required_monthly_revenue", v)} />
                  <PolicyRow label="Min Avg Balance (kobo)" value={form.policy.required_avg_balance}
                    hint={fmt(form.policy.required_avg_balance)} onChange={(v) => setPolicyField("required_avg_balance", v)} />
                  <PolicyRow label="Min Positive Cash Flow Months" value={form.policy.required_positive_cash_flow_months}
                    onChange={(v) => setPolicyField("required_positive_cash_flow_months", v)} />
                  <PolicyRow label="Max Revenue Volatility (bps)" value={form.policy.max_revenue_volatility_bps}
                    hint={`${(form.policy.max_revenue_volatility_bps / 100).toFixed(0)}%`} onChange={(v) => setPolicyField("max_revenue_volatility_bps", v)} />
                  <PolicyRow label="Max Debt Ratio (bps)" value={form.policy.max_debt_ratio_bps}
                    hint={`${(form.policy.max_debt_ratio_bps / 100).toFixed(0)}%`} onChange={(v) => setPolicyField("max_debt_ratio_bps", v)} />
                  <PolicyRow label="Max Customer Concentration (bps)" value={form.policy.max_customer_concentration_bps}
                    hint={`${(form.policy.max_customer_concentration_bps / 100).toFixed(0)}%`} onChange={(v) => setPolicyField("max_customer_concentration_bps", v)} />
                  <PolicyRow label="Min Account Age (months)" value={form.policy.required_account_age_months}
                    onChange={(v) => setPolicyField("required_account_age_months", v)} />
                </CardContent>
              </Card>
            </div>

            <div className="flex items-center gap-3 flex-wrap">
              <Button onClick={handleSave} disabled={saving} className="gap-2 bg-blue-600 hover:bg-blue-500">
                {saving ? <Loader2 className="h-4 w-4 animate-spin" /> : null}Save Profile
              </Button>
              {!profile?.published && (
                <Button variant="outline" onClick={async () => { await handleSave(); await handleTogglePublish(); }}
                  className="gap-2 border-green-800 text-green-400 hover:bg-green-950" disabled={saving || toggling}>
                  <Globe className="h-4 w-4" />Save & Publish
                </Button>
              )}
              <Button variant="ghost" size="sm"
                onClick={() => { setForm((f) => ({ ...f, policy: { ...DEFAULT_POLICY } })); setSaveMsg("Defaults loaded — click Save to apply."); }}
                className="text-slate-500 hover:text-slate-300 text-xs">
                Reset to defaults
              </Button>
              {saveMsg && <span className="text-sm text-green-400">{saveMsg}</span>}
            </div>
          </div>
        )}

        {/* ── Applications tab ── */}
        {tab === "applications" && (
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-xl font-semibold">Incoming Applications</h2>
                <p className="text-slate-400 text-sm mt-1">
                  No financial statements visible. Generate a ZK proof to evaluate eligibility cryptographically.
                </p>
              </div>
              <Button variant="ghost" size="sm" onClick={loadApps} className="text-slate-400 gap-1">
                <RefreshCw className="h-4 w-4" /> Refresh
              </Button>
            </div>

            {loadingApps ? (
              <div className="flex justify-center py-12"><Loader2 className="h-7 w-7 animate-spin text-slate-500" /></div>
            ) : applications.length === 0 ? (
              <Card className="bg-slate-900/50 border-slate-800 border-dashed text-center py-16">
                <Eye className="h-10 w-10 text-slate-600 mx-auto mb-3" />
                <p className="text-slate-500">No applications yet</p>
                <p className="text-slate-600 text-sm mt-1">Publish your profile so borrowers can apply</p>
              </Card>
            ) : (
              <div className="grid gap-4">
                {applications.map((app) => {
                  const result = results[app.id];
                  const isVerifying = verifying === app.id;
                  const isExpanded = expanded[app.id];

                  return (
                    <Card key={app.id} className={`border transition-colors ${
                      result?.status === "approved" ? "bg-slate-900 border-green-900/50" :
                      result?.status === "rejected" ? "bg-slate-900 border-red-900/50" :
                      result?.error ? "bg-slate-900 border-amber-900/50" :
                      "bg-slate-900 border-slate-800"
                    }`}>
                      <CardContent className="pt-5 space-y-0">
                        {/* Header row */}
                        <div className="flex items-center justify-between gap-4">
                          <div className="flex items-center gap-3">
                            <StatusIcon status={result ? result.status : app.status} />
                            <div>
                              <div className="font-semibold text-white text-sm">{app.borrower_ref ?? "Applicant"}</div>
                              <p className="text-xs text-slate-500">
                                Applied {new Date(app.created_at).toLocaleString()}
                                {app.decided_at && ` · Decided ${new Date(app.decided_at).toLocaleString()}`}
                              </p>
                            </div>
                          </div>
                          <div className="flex items-center gap-2 shrink-0">
                            <StatusBadge status={result ? result.status : app.status} />
                            {app.status === "pending" && !result && (
                              <Button size="sm" onClick={() => handleVerify(app.id)} disabled={isVerifying}
                                className="gap-1.5 bg-purple-700 hover:bg-purple-600 font-mono text-xs">
                                {isVerifying ? <><Loader2 className="h-3.5 w-3.5 animate-spin" /> Proving…</> : <><Zap className="h-3.5 w-3.5" /> Generate Proof</>}
                              </Button>
                            )}
                            {(result || isVerifying) && (
                              <button onClick={() => setExpanded((e) => ({ ...e, [app.id]: !e[app.id] }))}
                                className="text-slate-400 hover:text-white transition-colors p-1">
                                {isExpanded ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
                              </button>
                            )}
                          </div>
                        </div>

                        {/* ── Proof panel ── */}
                        {(isVerifying || result) && isExpanded && (
                          <div className="mt-4 border-t border-slate-800 pt-4 space-y-4">

                            {/* Step progress */}
                            <div className="space-y-2">
                              <p className="text-xs font-mono text-slate-500 uppercase tracking-widest mb-3">
                                Proof Generation Pipeline
                              </p>
                              {PROOF_STEPS.map((step, idx) => {
                                const done = !isVerifying || idx < proofStep;
                                const active = isVerifying && idx === proofStep;
                                return (
                                  <div key={step.id} className={`flex items-start gap-3 transition-opacity ${
                                    done || active ? "opacity-100" : "opacity-30"
                                  }`}>
                                    <div className={`mt-0.5 flex-shrink-0 h-5 w-5 rounded-full flex items-center justify-center text-xs ${
                                      active ? "bg-purple-700 animate-pulse" :
                                      done ? "bg-green-800" : "bg-slate-800"
                                    }`}>
                                      {active ? <Loader2 className="h-3 w-3 animate-spin" /> :
                                       done ? <CheckCircle2 className="h-3 w-3 text-green-400" /> :
                                       <span className="text-slate-500">{idx + 1}</span>}
                                    </div>
                                    <div>
                                      <p className={`text-sm font-medium ${active ? "text-purple-300" : done ? "text-white" : "text-slate-500"}`}>
                                        {step.label}
                                      </p>
                                      <p className="text-xs text-slate-500 font-mono">{step.sub}</p>
                                    </div>
                                  </div>
                                );
                              })}
                            </div>

                            {/* Error state */}
                            {result?.error && (
                              <div className="bg-red-950/40 border border-red-900 rounded-lg p-4 flex items-start gap-3">
                                <AlertTriangle className="h-5 w-5 text-red-400 shrink-0 mt-0.5" />
                                <div>
                                  <p className="text-sm font-semibold text-red-300 mb-1">Proof generation failed</p>
                                  <p className="text-xs font-mono text-red-400 break-all">{result.error}</p>
                                </div>
                              </div>
                            )}

                            {/* Proof details */}
                            {result && !result.error && (
                              <>
                                {/* Verdict banner */}
                                <div className={`rounded-lg p-4 border text-center ${
                                  result.status === "approved"
                                    ? "bg-green-950/40 border-green-800"
                                    : "bg-red-950/40 border-red-800"
                                }`}>
                                  <div className={`text-2xl font-black tracking-widest mb-1 ${
                                    result.status === "approved" ? "text-green-300" : "text-red-300"
                                  }`}>
                                    {result.status === "approved" ? "✓ LOAN APPROVED" : "✗ LOAN REJECTED"}
                                  </div>
                                  <p className="text-sm text-slate-400">{result.decision_reason}</p>
                                </div>

                                {/* Predicates */}
                                <div>
                                  <p className="text-xs font-mono text-slate-500 uppercase tracking-widest mb-2">
                                    ZK Predicate Verification
                                  </p>
                                  <div className="grid gap-2">
                                    {result.predicates.map((p) => (
                                      <div key={p.name} className={`flex items-center justify-between px-3 py-2 rounded-lg border text-sm ${
                                        p.satisfied
                                          ? "bg-green-950/30 border-green-900/60 text-green-200"
                                          : "bg-red-950/30 border-red-900/60 text-red-200"
                                      }`}>
                                        <div className="flex items-center gap-2">
                                          {p.satisfied
                                            ? <CheckCircle2 className="h-4 w-4 text-green-400 shrink-0" />
                                            : <XCircle className="h-4 w-4 text-red-400 shrink-0" />}
                                          <span>{p.description}</span>
                                        </div>
                                        <span className={`font-mono font-bold text-xs ${p.satisfied ? "text-green-400" : "text-red-400"}`}>
                                          {p.satisfied ? "PASS" : "FAIL"}
                                        </span>
                                      </div>
                                    ))}
                                  </div>
                                </div>

                                {/* Cryptographic proof metadata */}
                                {result.proof && (
                                  <div className="bg-slate-950 border border-slate-800 rounded-lg p-4 space-y-3">
                                    <p className="text-xs font-mono text-slate-500 uppercase tracking-widest">
                                      Cryptographic Proof Metadata
                                    </p>
                                    <div className="grid gap-2 text-xs font-mono">
                                      <MetaRow icon={<FileCode2 className="h-3.5 w-3.5 text-blue-400" />}
                                        label="Circuit" value={result.proof.circuit_id} />
                                      <MetaRow icon={<Hash className="h-3.5 w-3.5 text-purple-400" />}
                                        label="Proof ID" value={result.proof.id} />
                                      <MetaRow icon={<Lock className="h-3.5 w-3.5 text-amber-400" />}
                                        label="Proof hash (32 bytes)"
                                        value={result.proof.proof_hash + "…"}
                                        mono copyable />
                                      <MetaRow icon={<Lock className="h-3.5 w-3.5 text-slate-400" />}
                                        label="VK hash (16 bytes)"
                                        value={result.proof.vk_hash + "…"}
                                        mono copyable />
                                      <MetaRow icon={<Shield className="h-3.5 w-3.5 text-green-400" />}
                                        label="Proof size"
                                        value={`${result.proof.proof_size_bytes.toLocaleString()} bytes`} />
                                      <MetaRow icon={<CheckCircle2 className="h-3.5 w-3.5 text-green-400" />}
                                        label="Cryptographic verification"
                                        value={result.proof_verified ? "✓ VALID — UltraHonk verified" : "✗ INVALID"}
                                        className={result.proof_verified ? "text-green-400" : "text-red-400"} />
                                    </div>

                                    {/* Public inputs */}
                                    <div className="pt-2 border-t border-slate-800">
                                      <p className="text-xs text-slate-600 uppercase tracking-widest mb-2">
                                        Public Inputs (lender thresholds committed to circuit)
                                      </p>
                                      <div className="grid grid-cols-2 gap-1 text-xs font-mono text-slate-400">
                                        {Object.entries(result.proof.public_inputs).map(([k, v]) => (
                                          <div key={k} className="flex justify-between gap-2">
                                            <span className="text-slate-600 truncate">{k.replace(/_/g, "_")}</span>
                                            <span className="text-slate-300 shrink-0">{String(v)}</span>
                                          </div>
                                        ))}
                                      </div>
                                    </div>
                                  </div>
                                )}
                              </>
                            )}
                          </div>
                        )}

                        {/* Collapsed summary for already-decided apps */}
                        {result && !result.error && !isExpanded && (
                          <div className="mt-3 flex items-center gap-2 flex-wrap">
                            {result.predicates.map((p) => (
                              <span key={p.name} className={`text-xs px-2 py-0.5 rounded-full border ${
                                p.satisfied ? "border-green-900 text-green-400" : "border-red-900 text-red-400"
                              }`}>
                                {p.satisfied ? "✓" : "✗"} {p.name}
                              </span>
                            ))}
                          </div>
                        )}
                      </CardContent>
                    </Card>
                  );
                })}
              </div>
            )}
          </div>
        )}
      </main>
    </div>
  );
}

// ── Sub-components ────────────────────────────────────────────────────────────

function PolicyRow({ label, value, hint, onChange }: {
  label: string; value: number; hint?: string; onChange: (v: string) => void;
}) {
  return (
    <div className="flex items-center gap-3">
      <Label className="text-slate-400 text-xs flex-1 min-w-0">{label}</Label>
      <div className="flex items-center gap-2 shrink-0">
        {hint && <span className="text-xs text-blue-400 w-24 text-right">{hint}</span>}
        <Input type="number" value={value} onChange={(e) => onChange(e.target.value)}
          className="bg-slate-800 border-slate-700 text-white text-xs h-7 w-32" />
      </div>
    </div>
  );
}

function MetaRow({ icon, label, value, mono, copyable, className }: {
  icon: React.ReactNode; label: string; value: string;
  mono?: boolean; copyable?: boolean; className?: string;
}) {
  return (
    <div className="flex items-start gap-2">
      <span className="mt-0.5 shrink-0">{icon}</span>
      <span className="text-slate-500 shrink-0 w-44">{label}</span>
      <span
        className={`break-all ${mono ? "font-mono text-slate-300" : "text-slate-300"} ${className ?? ""} ${copyable ? "cursor-pointer hover:text-white" : ""}`}
        onClick={copyable ? () => navigator.clipboard.writeText(value) : undefined}
        title={copyable ? "Click to copy" : undefined}
      >
        {value}
      </span>
    </div>
  );
}

function StatusIcon({ status }: { status: string }) {
  if (status === "approved") return <CheckCircle2 className="h-5 w-5 text-green-400 shrink-0" />;
  if (status === "rejected") return <XCircle className="h-5 w-5 text-red-400 shrink-0" />;
  if (status === "error") return <AlertTriangle className="h-5 w-5 text-amber-400 shrink-0" />;
  return <Clock className="h-5 w-5 text-amber-400 shrink-0" />;
}

function StatusBadge({ status }: { status: string }) {
  const map: Record<string, string> = {
    approved: "bg-green-950 text-green-300 border-green-800",
    rejected: "bg-red-950 text-red-300 border-red-800",
    pending:  "bg-amber-950 text-amber-300 border-amber-800",
    error:    "bg-amber-950 text-amber-300 border-amber-800",
  };
  return (
    <span className={`text-xs border rounded-full px-2.5 py-1 capitalize font-medium ${map[status] ?? ""}`}>
      {status}
    </span>
  );
}
