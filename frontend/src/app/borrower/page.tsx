"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import {
  Shield, Upload, FileText, BarChart3, Building2, CheckCircle2,
  XCircle, Clock, ArrowRight, Loader2, LogOut, RefreshCw, AlertCircle,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { clearAuth, getUser } from "@/lib/auth";
import type { StoredUser } from "@/lib/auth";
import {
  uploadStatement, computeMetrics, getLatestMetrics,
  getPublishedLenders, createApplication, getMyApplications,
  getMe, updateStellarAddress,
} from "@/lib/api";
import type { MetricsSummary, LenderProfile, LoanApplication } from "@/lib/types";

type Tab = "statements" | "lenders" | "applications";

function fmt(n: number) {
  return `₦${(n / 100).toLocaleString("en-NG", { maximumFractionDigits: 0 })}`;
}
function pct(bps: number) {
  return `${(bps / 100).toFixed(1)}%`;
}

export default function BorrowerDashboard() {
  const router = useRouter();
  const [user, setUser] = useState<StoredUser | null>(null);
  const [tab, setTab] = useState<Tab>("statements");

  // statements tab
  const [uploading, setUploading] = useState(false);
  const [computing, setComputing] = useState(false);
  const [uploadMsg, setUploadMsg] = useState("");
  const [metrics, setMetrics] = useState<MetricsSummary | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  // lenders tab
  const [lenders, setLenders] = useState<LenderProfile[]>([]);
  const [loadingLenders, setLoadingLenders] = useState(false);
  const [applying, setApplying] = useState<string | null>(null);
  const [applyMsg, setApplyMsg] = useState<Record<string, string>>({});

  // applications tab
  const [applications, setApplications] = useState<LoanApplication[]>([]);
  const [loadingApps, setLoadingApps] = useState(false);

  // stellar address
  const [stellarAddress, setStellarAddress] = useState("");
  const [savingAddr, setSavingAddr] = useState(false);
  const [addrMsg, setAddrMsg] = useState("");

  useEffect(() => {
    const u = getUser();
    if (!u || u.role !== "borrower") {
      router.replace("/login");
      return;
    }
    setUser(u);
    loadMetrics();
    loadLenders();
    loadApps();
    getMe().then(m => setStellarAddress(m.stellar_address ?? "")).catch(() => {});
  }, []);

  async function loadMetrics() {
    try {
      setMetrics(await getLatestMetrics());
    } catch { /* no metrics yet */ }
  }

  async function loadLenders() {
    setLoadingLenders(true);
    try { setLenders(await getPublishedLenders()); } catch { /* ok */ }
    finally { setLoadingLenders(false); }
  }

  const loadApps = useCallback(async () => {
    setLoadingApps(true);
    try { setApplications(await getMyApplications()); } catch { /* ok */ }
    finally { setLoadingApps(false); }
  }, []);

  async function handleUpload(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0];
    if (!file) return;
    setUploading(true);
    setUploadMsg("");
    try {
      const res = await uploadStatement(file);
      setUploadMsg(`Statement uploaded (${res.statement_id.slice(0, 8)}…). Now compute metrics.`);
    } catch (err) {
      setUploadMsg(err instanceof Error ? err.message : "Upload failed");
    } finally {
      setUploading(false);
      if (fileRef.current) fileRef.current.value = "";
    }
  }

  async function handleCompute() {
    setComputing(true);
    try {
      const m = await computeMetrics();
      setMetrics(m);
      setUploadMsg("Metrics computed successfully.");
    } catch (err) {
      setUploadMsg(err instanceof Error ? err.message : "Failed");
    } finally {
      setComputing(false);
    }
  }

  async function handleSaveAddr() {
    setSavingAddr(true); setAddrMsg("");
    try {
      await updateStellarAddress(stellarAddress.trim());
      setAddrMsg("Saved.");
    } catch (err) {
      setAddrMsg(err instanceof Error ? err.message : "Failed");
    } finally { setSavingAddr(false); }
  }

  async function handleApply(lender: LenderProfile) {
    if (!metrics) {
      setApplyMsg({ ...applyMsg, [lender.id]: "Compute your metrics first." });
      return;
    }
    setApplying(lender.id);
    try {
      await createApplication({ lender_profile_id: lender.id, metrics_id: metrics.metrics_id });
      setApplyMsg({ ...applyMsg, [lender.id]: "Application submitted!" });
      loadApps();
    } catch (err) {
      setApplyMsg({ ...applyMsg, [lender.id]: err instanceof Error ? err.message : "Failed" });
    } finally {
      setApplying(null);
    }
  }

  function logout() {
    clearAuth();
    router.push("/");
  }

  const TABS: { id: Tab; label: string; icon: React.ReactNode }[] = [
    { id: "statements", label: "My Statement", icon: <FileText className="h-4 w-4" /> },
    { id: "lenders", label: "Browse Lenders", icon: <Building2 className="h-4 w-4" /> },
    { id: "applications", label: "Applications", icon: <BarChart3 className="h-4 w-4" /> },
  ];

  return (
    <div className="min-h-screen bg-slate-950 text-white">
      {/* Header */}
      <header className="border-b border-slate-800 bg-slate-950/80 backdrop-blur sticky top-0 z-10">
        <div className="max-w-5xl mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-blue-400" />
            <span className="font-bold tracking-tight">LedgerProof</span>
            <Badge variant="outline" className="ml-2 border-slate-700 text-slate-400 text-xs">
              Borrower
            </Badge>
          </div>
          <div className="flex items-center gap-3">
            <span className="text-sm text-slate-400">{user?.full_name ?? user?.username}</span>
            <button onClick={logout} className="text-slate-400 hover:text-white transition-colors">
              <LogOut className="h-4 w-4" />
            </button>
          </div>
        </div>
        {/* Tabs */}
        <div className="max-w-5xl mx-auto px-6 flex gap-1 pb-0">
          {TABS.map((t) => (
            <button
              key={t.id}
              onClick={() => setTab(t.id)}
              className={`flex items-center gap-2 px-4 py-2.5 text-sm font-medium border-b-2 transition-all ${
                tab === t.id
                  ? "border-blue-500 text-white"
                  : "border-transparent text-slate-400 hover:text-slate-200"
              }`}
            >
              {t.icon}
              {t.label}
              {t.id === "applications" && applications.length > 0 && (
                <span className="bg-slate-700 text-slate-300 text-xs rounded-full px-1.5 py-0.5">
                  {applications.length}
                </span>
              )}
            </button>
          ))}
        </div>
      </header>

      <main className="max-w-5xl mx-auto px-6 py-8">
        {/* ── Statements tab ── */}
        {tab === "statements" && (
          <div className="space-y-6">
            <div>
              <h2 className="text-xl font-semibold">Your Bank Statement</h2>
              <p className="text-slate-400 text-sm mt-1">
                Upload your statement — it is parsed locally, never shared with lenders.
              </p>
            </div>

            <div className="grid md:grid-cols-2 gap-6">
              <Card className="bg-slate-900 border-slate-800">
                <CardHeader>
                  <CardTitle className="text-base text-white">Upload Statement</CardTitle>
                  <CardDescription className="text-slate-400">
                    Supported format: XLSX
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div
                    onClick={() => fileRef.current?.click()}
                    className="border-2 border-dashed border-slate-700 rounded-lg p-8 text-center cursor-pointer hover:border-blue-600 transition-colors group"
                  >
                    <Upload className="h-8 w-8 text-slate-500 group-hover:text-blue-400 mx-auto mb-3 transition-colors" />
                    <p className="text-slate-400 text-sm">Click to select XLSX file</p>
                    <input ref={fileRef} type="file" accept=".xlsx" className="hidden" onChange={handleUpload} />
                  </div>

                  <Button onClick={handleCompute} disabled={computing} className="w-full gap-2">
                    {computing ? (
                      <><Loader2 className="h-4 w-4 animate-spin" /> Computing metrics…</>
                    ) : (
                      <><BarChart3 className="h-4 w-4" /> Compute Metrics</>
                    )}
                  </Button>

                  {uploading && (
                    <div className="flex items-center gap-2 text-sm text-slate-400">
                      <Loader2 className="h-4 w-4 animate-spin" /> Uploading…
                    </div>
                  )}
                  {uploadMsg && (
                    <p className={`text-sm ${uploadMsg.includes("failed") || uploadMsg.includes("Error") ? "text-red-400" : "text-green-400"}`}>
                      {uploadMsg}
                    </p>
                  )}
                </CardContent>
              </Card>

              {metrics ? (
                <Card className="bg-slate-900 border-slate-800">
                  <CardHeader>
                    <CardTitle className="text-base text-white">Financial Summary</CardTitle>
                    <CardDescription className="text-slate-400 text-xs">
                      Computed from your statement — not visible to lenders
                    </CardDescription>
                  </CardHeader>
                  <CardContent className="space-y-2.5">
                    {[
                      ["Avg Monthly Revenue", fmt(metrics.summary.avg_monthly_revenue_naira * 100)],
                      ["Avg Monthly Balance", fmt(metrics.summary.avg_monthly_balance_naira * 100)],
                      ["Positive Cash Flow Months", `${metrics.summary.positive_cash_flow_months} months`],
                      ["Revenue Volatility", pct(metrics.summary.revenue_volatility_pct * 100)],
                      ["Customer Concentration", pct(metrics.summary.customer_concentration_pct * 100)],
                      ["Debt Ratio", pct(metrics.summary.debt_ratio_pct * 100)],
                      ["Missed Repayments", metrics.summary.has_missed_repayments ? "Yes" : "None"],
                      ["Account Age", `${metrics.summary.account_age_months} months`],
                    ].map(([label, value]) => (
                      <div key={label} className="flex items-center justify-between text-sm">
                        <span className="text-slate-400">{label}</span>
                        <span className="text-white font-medium">{value}</span>
                      </div>
                    ))}
                  </CardContent>
                </Card>
              ) : (
                <Card className="bg-slate-900/50 border-slate-800 border-dashed flex items-center justify-center p-10">
                  <div className="text-center">
                    <BarChart3 className="h-10 w-10 text-slate-600 mx-auto mb-3" />
                    <p className="text-slate-500 text-sm">Metrics will appear here after computation</p>
                  </div>
                </Card>
              )}
            </div>

            {metrics && (
              <div className="bg-blue-950/30 border border-blue-900 rounded-lg px-4 py-3 flex items-center gap-3">
                <CheckCircle2 className="h-5 w-5 text-blue-400 shrink-0" />
                <p className="text-sm text-blue-200">
                  Metrics ready. Browse lenders and apply — they will only see ZK proof results, never your statement.
                </p>
                <Button size="sm" variant="outline" className="ml-auto border-blue-800 text-blue-300 hover:bg-blue-900" onClick={() => setTab("lenders")}>
                  Browse Lenders <ArrowRight className="h-3.5 w-3.5 ml-1" />
                </Button>
              </div>
            )}
          {/* Stellar receiving address */}
          <div className="rounded-xl border border-slate-700 bg-slate-900 p-5 space-y-3">
            <div className="flex items-center gap-2">
              <Shield className="h-4 w-4 text-blue-400" />
              <span className="font-medium text-sm text-white">Stellar Wallet Address</span>
              <span className="text-xs text-slate-500 ml-1">— for loan disbursement</span>
            </div>
            <p className="text-xs text-slate-400">
              When a lender approves your application, XLM is sent automatically to this address on Stellar testnet.
            </p>
            <div className="flex gap-2">
              <input
                className="flex-1 bg-slate-800 border border-slate-700 rounded-md px-3 py-2 text-sm text-white placeholder:text-slate-500 focus:outline-none focus:border-blue-600"
                placeholder="G... (Stellar public key)"
                value={stellarAddress}
                onChange={e => setStellarAddress(e.target.value)}
              />
              <Button size="sm" onClick={handleSaveAddr} disabled={savingAddr} className="bg-blue-600 hover:bg-blue-500">
                {savingAddr ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : "Save"}
              </Button>
            </div>
            {addrMsg && <p className="text-xs text-green-400">{addrMsg}</p>}
          </div>
        </div>
        )}

        {/* ── Lenders tab ── */}
        {tab === "lenders" && (
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-xl font-semibold">Browse Lenders</h2>
                <p className="text-slate-400 text-sm mt-1">
                  Select a lender whose criteria you want to apply against.
                </p>
              </div>
              <Button variant="ghost" size="sm" onClick={loadLenders} className="text-slate-400 gap-1">
                <RefreshCw className="h-4 w-4" /> Refresh
              </Button>
            </div>

            {!metrics && (
              <div className="bg-amber-950/30 border border-amber-900 rounded-lg px-4 py-3 flex items-center gap-3">
                <AlertCircle className="h-5 w-5 text-amber-400 shrink-0" />
                <p className="text-sm text-amber-200">
                  Upload and compute your metrics first so you can apply.
                </p>
                <Button size="sm" variant="outline" className="ml-auto border-amber-800 text-amber-300 hover:bg-amber-900" onClick={() => setTab("statements")}>
                  Upload Statement
                </Button>
              </div>
            )}

            {loadingLenders ? (
              <div className="flex justify-center py-12">
                <Loader2 className="h-7 w-7 animate-spin text-slate-500" />
              </div>
            ) : lenders.length === 0 ? (
              <Card className="bg-slate-900/50 border-slate-800 border-dashed text-center py-16">
                <Building2 className="h-10 w-10 text-slate-600 mx-auto mb-3" />
                <p className="text-slate-500">No lenders published yet</p>
              </Card>
            ) : (
              <div className="grid gap-4">
                {lenders.map((l) => (
                  <Card key={l.id} className="bg-slate-900 border-slate-800">
                    <CardContent className="pt-5">
                      <div className="flex items-start justify-between gap-4">
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2 mb-1">
                            <Building2 className="h-4 w-4 text-blue-400 shrink-0" />
                            <h3 className="font-semibold text-white truncate">{l.display_name}</h3>
                          </div>
                          {l.description && (
                            <p className="text-slate-400 text-sm mb-3">{l.description}</p>
                          )}
                          <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
                            {l.policy.required_monthly_revenue && (
                              <PolicyChip label="Min revenue" value={fmt(l.policy.required_monthly_revenue)} />
                            )}
                            {l.policy.required_avg_balance && (
                              <PolicyChip label="Min balance" value={fmt(l.policy.required_avg_balance)} />
                            )}
                            {l.policy.required_positive_cash_flow_months && (
                              <PolicyChip label="Cash flow" value={`≥ ${l.policy.required_positive_cash_flow_months}mo`} />
                            )}
                            {l.policy.max_revenue_volatility_bps && (
                              <PolicyChip label="Max volatility" value={pct(l.policy.max_revenue_volatility_bps)} />
                            )}
                            {l.policy.max_debt_ratio_bps && (
                              <PolicyChip label="Max debt ratio" value={pct(l.policy.max_debt_ratio_bps)} />
                            )}
                            {l.policy.max_customer_concentration_bps && (
                              <PolicyChip label="Max concentration" value={pct(l.policy.max_customer_concentration_bps)} />
                            )}
                          </div>
                        </div>
                        <div className="flex flex-col items-end gap-2">
                          <Button
                            size="sm"
                            disabled={!metrics || applying === l.id}
                            onClick={() => handleApply(l)}
                            className="gap-1 whitespace-nowrap"
                          >
                            {applying === l.id ? (
                              <><Loader2 className="h-3.5 w-3.5 animate-spin" /> Applying…</>
                            ) : (
                              <>Apply <ArrowRight className="h-3.5 w-3.5" /></>
                            )}
                          </Button>
                          {applyMsg[l.id] && (
                            <p className={`text-xs ${applyMsg[l.id].includes("submitted") ? "text-green-400" : "text-red-400"}`}>
                              {applyMsg[l.id]}
                            </p>
                          )}
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </div>
        )}

        {/* ── Applications tab ── */}
        {tab === "applications" && (
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-xl font-semibold">My Applications</h2>
                <p className="text-slate-400 text-sm mt-1">Track the status of your loan applications.</p>
              </div>
              <Button variant="ghost" size="sm" onClick={loadApps} className="text-slate-400 gap-1">
                <RefreshCw className="h-4 w-4" /> Refresh
              </Button>
            </div>

            {loadingApps ? (
              <div className="flex justify-center py-12">
                <Loader2 className="h-7 w-7 animate-spin text-slate-500" />
              </div>
            ) : applications.length === 0 ? (
              <Card className="bg-slate-900/50 border-slate-800 border-dashed text-center py-16">
                <FileText className="h-10 w-10 text-slate-600 mx-auto mb-3" />
                <p className="text-slate-500 mb-4">No applications yet</p>
                <Button size="sm" onClick={() => setTab("lenders")} variant="outline" className="border-slate-700">
                  Browse Lenders
                </Button>
              </Card>
            ) : (
              <div className="grid gap-3">
                {applications.map((app) => (
                  <Card key={app.id} className={`border-slate-800 ${app.status === "approved" ? "bg-green-950/20" : "bg-slate-900"}`}>
                    <CardContent className="pt-4 space-y-3">
                      <div className="flex items-center gap-4">
                        <StatusIcon status={app.status} />
                        <div className="flex-1 min-w-0">
                          <div className="font-medium text-white text-sm">
                            {app.lender?.display_name ?? "Lender"}
                          </div>
                          {app.decision_reason && (
                            <p className="text-xs text-slate-400 mt-0.5">{app.decision_reason}</p>
                          )}
                          <p className="text-xs text-slate-500 mt-1">
                            Applied {new Date(app.created_at).toLocaleDateString()}
                            {app.decided_at && ` · Decided ${new Date(app.decided_at).toLocaleDateString()}`}
                          </p>
                        </div>
                        <StatusBadge status={app.status} />
                      </div>
                      {app.status === "approved" && (
                        <div className="rounded-lg bg-green-950/40 border border-green-800 px-4 py-3 space-y-1">
                          <p className="text-xs font-medium text-green-300 flex items-center gap-1.5">
                            <CheckCircle2 className="h-3.5 w-3.5" /> Loan Disbursed on Stellar
                          </p>
                          <p className="text-xs text-slate-400">
                            XLM has been sent to your Stellar wallet automatically via smart contract.
                          </p>
                          {!stellarAddress && (
                            <p className="text-xs text-amber-400">Add your Stellar address above to receive future disbursements.</p>
                          )}
                        </div>
                      )}
                    </CardContent>
                  </Card>
                ))}
              </div>
            )}
          </div>
        )}
      </main>
    </div>
  );
}

function PolicyChip({ label, value }: { label: string; value: string }) {
  return (
    <div className="bg-slate-800 rounded-md px-2.5 py-1.5 text-xs">
      <span className="text-slate-500">{label}: </span>
      <span className="text-slate-200 font-medium">{value}</span>
    </div>
  );
}

function StatusIcon({ status }: { status: string }) {
  if (status === "approved") return <CheckCircle2 className="h-5 w-5 text-green-400 shrink-0" />;
  if (status === "rejected") return <XCircle className="h-5 w-5 text-red-400 shrink-0" />;
  return <Clock className="h-5 w-5 text-amber-400 shrink-0" />;
}

function StatusBadge({ status }: { status: string }) {
  const map: Record<string, string> = {
    approved: "bg-green-950 text-green-300 border-green-800",
    rejected: "bg-red-950 text-red-300 border-red-800",
    pending:  "bg-amber-950 text-amber-300 border-amber-800",
  };
  return (
    <span className={`text-xs border rounded-full px-2.5 py-1 capitalize font-medium ${map[status] ?? ""}`}>
      {status}
    </span>
  );
}
