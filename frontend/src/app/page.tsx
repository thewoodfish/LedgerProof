import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Shield, FileText, BarChart3, Lock } from "lucide-react";

export default function Home() {
  return (
    <div className="min-h-screen bg-gradient-to-b from-slate-50 to-white">
      {/* Header */}
      <header className="border-b bg-white/80 backdrop-blur sticky top-0 z-10">
        <div className="max-w-6xl mx-auto px-6 py-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Shield className="h-6 w-6 text-blue-600" />
            <span className="font-bold text-lg">LedgerProof</span>
          </div>
          <nav className="flex gap-3">
            <Link href="/merchant">
              <Button variant="outline" size="sm">Merchant Portal</Button>
            </Link>
            <Link href="/lender">
              <Button size="sm">Lender Portal</Button>
            </Link>
          </nav>
        </div>
      </header>

      {/* Hero */}
      <section className="max-w-4xl mx-auto px-6 py-24 text-center">
        <div className="inline-flex items-center gap-2 bg-blue-50 text-blue-700 rounded-full px-4 py-1.5 text-sm font-medium mb-8">
          <Lock className="h-3.5 w-3.5" />
          Zero-Knowledge Proofs on Stellar
        </div>
        <h1 className="text-5xl font-bold text-slate-900 mb-6 leading-tight">
          Prove Financial Health.
          <br />
          <span className="text-blue-600">Not Financial History.</span>
        </h1>
        <p className="text-xl text-slate-500 mb-10 max-w-2xl mx-auto">
          LedgerProof lets SMEs prove they meet lending criteria using cryptographic proofs —
          without sharing a single bank statement.
        </p>
        <div className="flex gap-4 justify-center flex-wrap">
          <Link href="/merchant">
            <Button size="lg" className="gap-2">
              <FileText className="h-4 w-4" />
              I'm a Merchant
            </Button>
          </Link>
          <Link href="/lender">
            <Button size="lg" variant="outline" className="gap-2">
              <BarChart3 className="h-4 w-4" />
              I'm a Lender
            </Button>
          </Link>
        </div>
      </section>

      {/* How it works */}
      <section className="max-w-6xl mx-auto px-6 py-16">
        <h2 className="text-3xl font-bold text-center text-slate-900 mb-12">How it works</h2>
        <div className="grid md:grid-cols-3 gap-8">
          {[
            {
              step: "01",
              title: "Upload Statements",
              body: "Merchant uploads 6 months of bank statements once. They are parsed and never stored long-term.",
              icon: <FileText className="h-8 w-8 text-blue-600" />,
            },
            {
              step: "02",
              title: "Generate ZK Proof",
              body: "Financial metrics are computed and fed into a Noir circuit. The UltraHonk prover generates a cryptographic proof.",
              icon: <Lock className="h-8 w-8 text-purple-600" />,
            },
            {
              step: "03",
              title: "Instant Decision",
              body: "The lender's Soroban contract verifies the proof on-chain. Loan approved or declined — no statements viewed.",
              icon: <Shield className="h-8 w-8 text-green-600" />,
            },
          ].map((item) => (
            <Card key={item.step} className="relative overflow-hidden">
              <CardHeader>
                <div className="text-5xl font-black text-slate-100 absolute top-4 right-4">
                  {item.step}
                </div>
                {item.icon}
                <CardTitle className="mt-4">{item.title}</CardTitle>
              </CardHeader>
              <CardContent>
                <p className="text-slate-500 text-sm leading-relaxed">{item.body}</p>
              </CardContent>
            </Card>
          ))}
        </div>
      </section>

      {/* Metrics */}
      <section className="bg-slate-900 text-white py-16">
        <div className="max-w-6xl mx-auto px-6">
          <h2 className="text-2xl font-bold text-center mb-10 text-slate-200">
            14 Financial Metrics. Zero Data Exposed.
          </h2>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm text-slate-400">
            {[
              "Monthly Revenue", "Revenue Stability", "Positive Cash Flow", "Avg Balance",
              "Min Cash Reserve", "Revenue Growth", "Business Activity", "Customer Diversity",
              "Supplier Diversity", "Expense Stability", "Debt Ratio", "Loan Repayment History",
              "Account Age", "Transaction Frequency",
            ].map((m) => (
              <div key={m} className="flex items-center gap-2">
                <span className="w-1.5 h-1.5 rounded-full bg-blue-400 shrink-0" />
                {m}
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="border-t py-8 text-center text-sm text-slate-400">
        Built with Noir + Barretenberg + Soroban on Stellar
      </footer>
    </div>
  );
}
