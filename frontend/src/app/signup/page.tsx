"use client";

import { Suspense, useState } from "react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { Shield, Building2, User, Loader2, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { register } from "@/lib/api";
import { setAuth } from "@/lib/auth";
import { cn } from "@/lib/utils";

function SignupForm() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [role, setRole] = useState<"borrower" | "lender">(
    (searchParams.get("role") as "borrower" | "lender") ?? "borrower"
  );
  const [form, setForm] = useState({ username: "", password: "", full_name: "" });
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    if (form.password.length < 8) {
      setError("Password must be at least 8 characters");
      return;
    }
    setLoading(true);
    try {
      const { token, user } = await register({ ...form, role });
      setAuth(token, user);
      router.push(user.role === "lender" ? "/lender" : "/borrower");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Registration failed");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="min-h-screen bg-slate-950 flex items-center justify-center px-4 py-12">
      <div className="w-full max-w-md">
        <div className="flex items-center justify-center gap-2 mb-8">
          <Shield className="h-6 w-6 text-blue-400" />
          <span className="text-xl font-bold text-white tracking-tight">LedgerProof</span>
        </div>

        <Card className="bg-slate-900 border-slate-800">
          <CardHeader className="text-center">
            <CardTitle className="text-white">Create an account</CardTitle>
            <CardDescription className="text-slate-400">
              Join LedgerProof — privacy-preserving SME finance
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-6">
            {/* Role selector */}
            <div className="grid grid-cols-2 gap-3">
              {(
                [
                  {
                    value: "borrower",
                    label: "Borrower",
                    sub: "Apply for loans privately",
                    Icon: User,
                  },
                  {
                    value: "lender",
                    label: "Lender",
                    sub: "Set policy & review proofs",
                    Icon: Building2,
                  },
                ] as const
              ).map(({ value, label, sub, Icon }) => (
                <button
                  key={value}
                  type="button"
                  onClick={() => setRole(value)}
                  className={cn(
                    "flex flex-col items-start gap-1 p-4 rounded-lg border text-left transition-all",
                    role === value
                      ? "border-blue-500 bg-blue-950/50"
                      : "border-slate-700 bg-slate-800/50 hover:border-slate-600"
                  )}
                >
                  <div className="flex items-center justify-between w-full">
                    <Icon
                      className={cn(
                        "h-5 w-5",
                        role === value ? "text-blue-400" : "text-slate-400"
                      )}
                    />
                    {role === value && (
                      <Check className="h-4 w-4 text-blue-400" />
                    )}
                  </div>
                  <span
                    className={cn(
                      "font-semibold text-sm mt-1",
                      role === value ? "text-white" : "text-slate-300"
                    )}
                  >
                    {label}
                  </span>
                  <span className="text-xs text-slate-500">{sub}</span>
                </button>
              ))}
            </div>

            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-1.5">
                <Label htmlFor="full_name" className="text-slate-300">Full name</Label>
                <Input
                  id="full_name"
                  type="text"
                  placeholder="Your name"
                  value={form.full_name}
                  onChange={(e) => setForm({ ...form, full_name: e.target.value })}
                  className="bg-slate-800 border-slate-700 text-white placeholder:text-slate-500"
                />
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="username" className="text-slate-300">Username</Label>
                <Input
                  id="username"
                  type="text"
                  autoComplete="username"
                  placeholder="choose-a-username"
                  value={form.username}
                  onChange={(e) => setForm({ ...form, username: e.target.value })}
                  className="bg-slate-800 border-slate-700 text-white placeholder:text-slate-500"
                  required
                />
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="password" className="text-slate-300">Password</Label>
                <Input
                  id="password"
                  type="password"
                  autoComplete="new-password"
                  placeholder="Min. 8 characters"
                  value={form.password}
                  onChange={(e) => setForm({ ...form, password: e.target.value })}
                  className="bg-slate-800 border-slate-700 text-white placeholder:text-slate-500"
                  required
                />
              </div>

              {error && (
                <p className="text-sm text-red-400 bg-red-950/50 border border-red-900 rounded px-3 py-2">
                  {error}
                </p>
              )}

              <Button
                type="submit"
                disabled={loading}
                className="w-full bg-blue-600 hover:bg-blue-500"
              >
                {loading ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  `Create ${role} account`
                )}
              </Button>
            </form>

            <p className="text-center text-sm text-slate-500">
              Already have an account?{" "}
              <Link href="/login" className="text-blue-400 hover:text-blue-300 transition-colors">
                Sign in
              </Link>
            </p>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

export default function SignupPage() {
  return (
    <Suspense>
      <SignupForm />
    </Suspense>
  );
}
