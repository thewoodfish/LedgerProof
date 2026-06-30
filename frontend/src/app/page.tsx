"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { Shield, FileText, BarChart3, Lock, ArrowRight, CheckCircle2, LogOut } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { getUser, clearAuth } from "@/lib/auth";
import type { StoredUser } from "@/lib/auth";

export default function Home() {
  const router = useRouter();
  const [user, setUser] = useState<StoredUser | null>(null);

  useEffect(() => {
    setUser(getUser());
  }, []);

  function handleLogout() {
    clearAuth();
    setUser(null);
  }

  function goDashboard() {
    if (!user) return;
    router.push(user.role === "lender" ? "/lender" : "/borrower");
  }

  return (
    <div className="min-h-screen bg-gradient-to-b from-slate-950 via-slate-900 to-slate-950 text-white">
      {/* Nav */}
      <header className="border-b border-slate-800 sticky top-0 z-10 bg-slate-950/80 backdrop-blur">
        <div className="max-w-6xl mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-blue-400" />
            <span className="font-bold text-lg tracking-tight">LedgerProof</span>
          </div>
          <nav className="flex items-center gap-3">
            {user ? (
              <>
                <span className="text-sm text-slate-400">
                  {user.username} · <span className="capitalize">{user.role}</span>
                </span>
                <Button size="sm" onClick={goDashboard} className="gap-1">
                  Dashboard <ArrowRight className="h-3.5 w-3.5" />
                </Button>
                <button
                  onClick={handleLogout}
                  className="text-slate-400 hover:text-white p-1.5 rounded transition-colors"
                >
                  <LogOut className="h-4 w-4" />
                </button>
              </>
            ) : (
              <>
                <Link href="/login">
                  <Button variant="ghost" size="sm" className="text-slate-300 hover:text-white">
                    Sign in
                  </Button>
                </Link>
                <Link href="/signup">
                  <Button size="sm" className="bg-blue-600 hover:bg-blue-500">
                    Get started
                  </Button>
                </Link>
              </>
            )}
          </nav>
        </div>
      </header>

      {/* Hero */}
      <section className="max-w-4xl mx-auto px-6 pt-24 pb-20 text-center">
        <div className="inline-flex items-center gap-2 bg-blue-950 text-blue-300 border border-blue-800 rounded-full px-4 py-1.5 text-sm font-medium mb-8">
          <Lock className="h-3.5 w-3.5" />
          Zero-Knowledge Proofs · Stellar / Soroban
        </div>
        <h1 className="text-5xl sm:text-6xl font-extrabold mb-6 leading-tight tracking-tight">
          Prove financial health.
          <br />
          <span className="text-blue-400">Not financial history.</span>
        </h1>
        <p className="text-xl text-slate-400 mb-12 max-w-2xl mx-auto leading-relaxed">
          SMEs get loans without handing over bank statements.
          Lenders get cryptographic proof — not documents.
        </p>
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <Link href="/signup">
            <Button size="lg" className="gap-2 bg-blue-600 hover:bg-blue-500 w-full sm:w-auto">
              <FileText className="h-4 w-4" />
              Apply for a loan
            </Button>
          </Link>
          <Link href="/signup?role=lender">
            <Button
              size="lg"
              variant="outline"
              className="gap-2 border-slate-700 text-slate-300 hover:bg-slate-800 w-full sm:w-auto"
            >
              <BarChart3 className="h-4 w-4" />
              Become a lender
            </Button>
          </Link>
        </div>
      </section>

      {/* How it works */}
      <section className="max-w-6xl mx-auto px-6 py-20 border-t border-slate-800">
        <h2 className="text-3xl font-bold text-center mb-3">How it works</h2>
        <p className="text-slate-400 text-center mb-14 max-w-xl mx-auto">
          Three steps from statement to loan decision — no documents shared.
        </p>
        <div className="grid md:grid-cols-3 gap-6">
          {[
            {
              step: "01",
              title: "Upload your statement",
              body: "Upload your bank statement once. Transactions are parsed privately — they never leave your account.",
              icon: <FileText className="h-7 w-7 text-blue-400" />,
            },
            {
              step: "02",
              title: "Generate a ZK proof",
              body: "Your financial metrics go into a Noir circuit. The UltraHonk prover outputs a cryptographic proof of your eligibility.",
              icon: <Lock className="h-7 w-7 text-purple-400" />,
            },
            {
              step: "03",
              title: "Instant decision",
              body: "Lenders verify the proof on Soroban. Loan approved or declined automatically — statements never viewed.",
              icon: <Shield className="h-7 w-7 text-green-400" />,
            },
          ].map((item) => (
            <Card key={item.step} className="bg-slate-900 border-slate-800 relative overflow-hidden">
              <div className="text-6xl font-black text-slate-800 absolute top-3 right-4 select-none">
                {item.step}
              </div>
              <CardHeader className="pb-2">
                {item.icon}
                <CardTitle className="text-white mt-3 text-lg">{item.title}</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-slate-400 text-sm leading-relaxed">{item.body}</p>
              </CardContent>
            </Card>
          ))}
        </div>
      </section>

      {/* Privacy callout */}
      <section className="max-w-6xl mx-auto px-6 py-20 border-t border-slate-800">
        <div className="grid md:grid-cols-2 gap-12 items-center">
          <div>
            <h2 className="text-3xl font-bold mb-4">14 metrics. Zero documents.</h2>
            <p className="text-slate-400 mb-6 leading-relaxed">
              LedgerProof computes every underwriting signal lenders need — without
              exposing a single transaction, customer name, or balance figure.
            </p>
            <div className="grid grid-cols-2 gap-2 text-sm text-slate-400">
              {[
                "Monthly Revenue", "Revenue Stability", "Positive Cash Flow",
                "Avg Balance", "Min Cash Reserve", "Revenue Growth",
                "Business Activity", "Customer Diversity", "Supplier Diversity",
                "Expense Stability", "Debt Ratio", "Loan Repayment History",
                "Account Age", "Transaction Frequency",
              ].map((m) => (
                <div key={m} className="flex items-center gap-1.5">
                  <CheckCircle2 className="h-3.5 w-3.5 text-green-500 shrink-0" />
                  {m}
                </div>
              ))}
            </div>
          </div>
          <div className="bg-slate-900 border border-slate-800 rounded-2xl p-6 font-mono text-sm space-y-3">
            <div className="text-slate-500 text-xs mb-4">// Proof package — what the lender receives</div>
            {[
              ["Revenue ≥ ₦5M", true],
              ["Positive cash flow", true],
              ["Avg balance ≥ ₦500k", true],
              ["Volatility ≤ 15%", true],
              ["Customer conc. ≤ 25%", true],
              ["No missed repayments", true],
            ].map(([label, ok]) => (
              <div key={label as string} className="flex items-center justify-between">
                <span className="text-slate-300">{label as string}</span>
                <span className={ok ? "text-green-400" : "text-red-400"}>{ok ? "✓ TRUE" : "✗ FALSE"}</span>
              </div>
            ))}
            <div className="pt-3 border-t border-slate-800 text-slate-500 text-xs">
              Raw balances, transactions, customers — hidden.
            </div>
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="border-t border-slate-800 py-20 text-center">
        <h2 className="text-3xl font-bold mb-4">Ready to get started?</h2>
        <p className="text-slate-400 mb-8">Join as a borrower or set up your lending desk.</p>
        <div className="flex flex-col sm:flex-row gap-4 justify-center">
          <Link href="/signup">
            <Button size="lg" className="bg-blue-600 hover:bg-blue-500 gap-2">
              Apply for a loan <ArrowRight className="h-4 w-4" />
            </Button>
          </Link>
          <Link href="/signup?role=lender">
            <Button size="lg" variant="outline" className="border-slate-700 text-slate-300 hover:bg-slate-800">
              Become a lender
            </Button>
          </Link>
        </div>
      </section>

      <footer className="border-t border-slate-800 py-8 text-center text-sm text-slate-600">
        Built with Noir · Barretenberg · Soroban on Stellar
      </footer>
    </div>
  );
}
