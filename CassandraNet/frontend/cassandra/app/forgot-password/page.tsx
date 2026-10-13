"use client"

import type React from "react"

import { useState, useEffect } from "react"
import Link from "next/link"
import Image from "next/image"
import { Button } from "@/components/ui/button"
import { GridBackground } from "@/components/grid-background"
import { ArrowLeft, Mail, CheckCircle2, KeyRound } from "lucide-react"
import { motion, AnimatePresence } from "framer-motion"

export default function ForgotPasswordPage() {
  const [email, setEmail] = useState("")
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitted, setIsSubmitted] = useState(false)

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsLoading(true)
    // Simulate API call
    await new Promise((resolve) => setTimeout(resolve, 1500))
    setIsLoading(false)
    setIsSubmitted(true)
  }

  return (
    <div className="relative min-h-screen flex bg-background overflow-hidden">
      {/* Background */}
      <GridBackground />

      {/* Subtle ambient glow */}
      <div
        className="absolute top-1/4 left-1/2 -translate-x-1/2 w-[600px] h-[300px] rounded-full pointer-events-none"
        style={{
          background: "radial-gradient(ellipse, rgba(34, 94, 223, 0.08) 0%, transparent 70%)",
        }}
      />

      {/* Left side - Branding */}
      <div className="hidden lg:flex lg:w-1/2 relative z-10 flex-col justify-between p-12">
        {/* Logo */}
        <Link href="/" className="flex items-center">
          <Image src="/cass-logo-small.svg" alt="Infiner" width={138} height={32} priority />
        </Link>

        {/* Security illustration / messaging */}
        <div className="max-w-md">
          <div className="mb-8">
            <div className="w-20 h-20 rounded-2xl bg-primary/10 flex items-center justify-center mb-6">
              <KeyRound className="w-10 h-10 text-primary" />
            </div>
            <h2 className="text-3xl font-semibold text-foreground mb-4">Secure account recovery</h2>
            <p className="text-muted-foreground text-lg leading-relaxed">
              We take security seriously. Your password reset link will expire in 1 hour and can only be used once.
            </p>
          </div>

          {/* Security features */}
          <div className="space-y-4">
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0">
                <CheckCircle2 className="w-4 h-4 text-primary" />
              </div>
              <span className="text-foreground">End-to-end encrypted communications</span>
            </div>
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0">
                <CheckCircle2 className="w-4 h-4 text-primary" />
              </div>
              <span className="text-foreground">Single-use recovery links</span>
            </div>
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center flex-shrink-0">
                <CheckCircle2 className="w-4 h-4 text-primary" />
              </div>
              <span className="text-foreground">SOC 2 Type II certified infrastructure</span>
            </div>
          </div>
        </div>

        {/* Stats */}
        <div className="flex gap-12">
          <div>
            <p className="text-3xl font-semibold text-foreground">256-bit</p>
            <p className="text-sm text-muted-foreground mt-1">Encryption</p>
          </div>
          <div>
            <p className="text-3xl font-semibold text-foreground">99.99%</p>
            <p className="text-sm text-muted-foreground mt-1">Uptime</p>
          </div>
          <div>
            <p className="text-3xl font-semibold text-foreground">24/7</p>
            <p className="text-sm text-muted-foreground mt-1">Support</p>
          </div>
        </div>
      </div>

      {/* Right side - Form */}
      <div className="w-full lg:w-1/2 relative z-10 flex items-center justify-center p-6 lg:p-12">
        <div className="w-full max-w-md">
          {/* Mobile logo */}
          <div className="lg:hidden mb-8">
            <Link href="/" className="flex items-center justify-center">
              <Image src="/cass-logo-small.svg" alt="Infiner" width={138} height={32} priority />
            </Link>
          </div>

          {/* Form card */}
          <div className="bg-card/50 backdrop-blur-sm border border-white/5 rounded-xl p-8">
            <AnimatePresence mode="wait">
              {!isSubmitted ? (
                <motion.div
                  key="form"
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -10 }}
                  transition={{ duration: 0.2 }}
                >
                  {/* Back link */}
                  <Link
                    href="/login"
                    className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors mb-6"
                  >
                    <ArrowLeft className="w-4 h-4" />
                    Back to sign in
                  </Link>

                  <div className="mb-8">
                    <h1 className="text-2xl font-semibold text-foreground">Reset your password</h1>
                    <p className="text-muted-foreground mt-2 text-sm">
                      Enter your email address and we'll send you a link to reset your password.
                    </p>
                  </div>

                  {/* Form */}
                  <form onSubmit={handleSubmit} className="space-y-4">
                    <div>
                      <label htmlFor="email" className="block text-sm font-medium text-foreground mb-2">
                        Email address
                      </label>
                      <div className="relative">
                        <input
                          id="email"
                          type="email"
                          value={email}
                          onChange={(e) => setEmail(e.target.value)}
                          placeholder="you@company.com"
                          required
                          className="w-full px-4 py-2.5 pl-11 rounded-lg bg-white/5 border border-white/10 text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-all"
                        />
                        <Mail className="absolute left-4 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground" />
                      </div>
                    </div>

                    <Button
                      type="submit"
                      className="w-full bg-primary hover:bg-primary/90 text-primary-foreground py-2.5 mt-2"
                      disabled={isLoading}
                    >
                      {isLoading ? (
                        <span className="flex items-center gap-2">
                          <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                            <circle
                              className="opacity-25"
                              cx="12"
                              cy="12"
                              r="10"
                              stroke="currentColor"
                              strokeWidth="4"
                              fill="none"
                            />
                            <path
                              className="opacity-75"
                              fill="currentColor"
                              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                            />
                          </svg>
                          Sending link...
                        </span>
                      ) : (
                        "Send reset link"
                      )}
                    </Button>
                  </form>
                </motion.div>
              ) : (
                <motion.div
                  key="success"
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -10 }}
                  transition={{ duration: 0.2 }}
                  className="text-center py-4"
                >
                  {/* Success state */}
                  <div className="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center mx-auto mb-6">
                    <motion.div
                      initial={{ scale: 0 }}
                      animate={{ scale: 1 }}
                      transition={{ delay: 0.2, type: "spring", stiffness: 200 }}
                    >
                      <CheckCircle2 className="w-8 h-8 text-primary" />
                    </motion.div>
                  </div>

                  <h1 className="text-2xl font-semibold text-foreground mb-2">Check your email</h1>
                  <p className="text-muted-foreground text-sm mb-6">
                    We've sent a password reset link to <span className="text-foreground font-medium">{email}</span>
                  </p>

                  <div className="bg-white/5 border border-white/10 rounded-lg p-4 mb-6">
                    <p className="text-sm text-muted-foreground">
                      Didn't receive the email? Check your spam folder or{" "}
                      <button onClick={() => setIsSubmitted(false)} className="text-primary hover:underline">
                        try another email address
                      </button>
                    </p>
                  </div>

                  <Link href="/login">
                    <Button variant="outline" className="w-full border-white/10 hover:bg-white/5 bg-transparent">
                      <ArrowLeft className="w-4 h-4 mr-2" />
                      Back to sign in
                    </Button>
                  </Link>
                </motion.div>
              )}
            </AnimatePresence>
          </div>

          {/* Help link */}
          <p className="mt-6 text-center text-sm text-muted-foreground">
            Need help?{" "}
            <Link href="/contact" className="text-primary hover:underline font-medium">
              Contact support
            </Link>
          </p>
        </div>
      </div>
    </div>
  )
}
