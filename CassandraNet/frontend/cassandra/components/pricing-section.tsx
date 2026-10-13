"use client"

import { useEffect, useRef, useState } from "react"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Check } from "lucide-react"
import Link from "next/link"

export function PricingSection() {
  const [isVisible, setIsVisible] = useState(false)
  const sectionRef = useRef<HTMLElement>(null)

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setIsVisible(true)
          observer.disconnect()
        }
      },
      { threshold: 0.2 },
    )

    if (sectionRef.current) {
      observer.observe(sectionRef.current)
    }

    return () => observer.disconnect()
  }, [])

  const developerFeatures = [
    "Access to 50+ models",
    "Pay per token pricing",
    "Standard rate limits",
    "Community support",
    "Usage analytics dashboard",
    "API playground access",
  ]

  const enterpriseFeatures = [
    "Everything in Developer",
    "Custom model fine-tuning",
    "99.99% uptime SLA",
    "Dedicated support engineer",
    "Unlimited rate limits",
    "VPC deployment options",
  ]

  return (
    <section ref={sectionRef} id="pricing" className="relative py-24 md:py-32 overflow-hidden bg-background">
      <div className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[1000px] h-[200px] rounded-full blur-[120px] pointer-events-none bg-primary/[0.08]" />

      <div className="w-full flex justify-center px-4">
        <div className="flex flex-col items-center gap-12 max-w-4xl w-full">
          <div className="text-center max-w-2xl">
            <h2
              className={`text-3xl md:text-4xl lg:text-5xl font-bold text-foreground mb-4 text-balance ${
                isVisible ? "animate-slide-up-section" : "opacity-0"
              }`}
            >
              Simple, transparent pricing
            </h2>
            <p
              className={`text-muted-foreground text-lg leading-relaxed ${
                isVisible ? "animate-slide-up-section-delayed" : "opacity-0"
              }`}
            >
              Start building for free, scale when you're ready. No hidden fees, no surprises.
            </p>
          </div>

          <div
            className={`grid md:grid-cols-2 gap-8 w-full ${
              isVisible ? "animate-slide-up-section-delayed" : "opacity-0"
            }`}
            style={{ animationDelay: "0.4s" }}
          >
            <Card className="bg-surface-elevated border-white/10 relative overflow-hidden flex flex-col">
              <div className="absolute inset-0 bg-gradient-to-br from-primary/5 to-transparent pointer-events-none" />
              <CardHeader className="relative pb-0">
                <div className="w-12 h-12 rounded-xl bg-primary/10 flex items-center justify-center mb-4">
                  <svg
                    className="w-6 h-6 text-primary"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.5"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
                  </svg>
                </div>
                <CardTitle className="text-2xl text-foreground">Developer</CardTitle>
                <CardDescription className="text-muted-foreground mt-2">
                  Perfect for indie hackers and small teams getting started.
                </CardDescription>
              </CardHeader>
              <CardContent className="relative flex flex-col flex-1 pt-6">
                <div className="mb-6">
                  <span className="text-xl font-semibold text-foreground">Pay as you go</span>
                  <p className="text-muted-foreground text-sm mt-1">Only pay for what you use</p>
                </div>
                <ul className="space-y-4 flex-1">
                  {developerFeatures.map((feature, index) => (
                    <li key={index} className="flex items-center gap-3 text-muted-foreground">
                      <div className="w-5 h-5 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
                        <Check className="h-3 w-3 text-primary" />
                      </div>
                      <span>{feature}</span>
                    </li>
                  ))}
                </ul>
                <Button asChild className="w-full bg-primary hover:bg-primary/90 text-primary-foreground mt-8">
                  <Link href="/signup">Get Started</Link>
                </Button>
              </CardContent>
            </Card>

            <Card className="bg-surface-elevated border-primary/30 relative overflow-hidden flex flex-col">
              <div className="absolute inset-0 bg-gradient-to-br from-primary/10 to-transparent pointer-events-none" />
              <CardHeader className="relative pb-0">
                <div className="w-12 h-12 rounded-xl bg-primary/10 flex items-center justify-center mb-4">
                  <svg
                    className="w-6 h-6 text-primary"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="1.5"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <path d="M3 21h18" />
                    <path d="M5 21V7l8-4v18" />
                    <path d="M19 21V11l-6-4" />
                    <path d="M9 9v.01" />
                    <path d="M9 12v.01" />
                    <path d="M9 15v.01" />
                    <path d="M9 18v.01" />
                  </svg>
                </div>
                <div className="flex items-center gap-3">
                  <CardTitle className="text-2xl text-foreground">Enterprise</CardTitle>
                  <span className="px-2.5 py-0.5 rounded-full bg-primary/20 text-primary text-xs font-medium">
                    Custom
                  </span>
                </div>
                <CardDescription className="text-muted-foreground mt-2">
                  For organizations that need advanced security and support.
                </CardDescription>
              </CardHeader>
              <CardContent className="relative flex flex-col flex-1 pt-6">
                <div className="mb-6">
                  <span className="text-xl font-semibold text-foreground">Custom pricing</span>
                  <p className="text-muted-foreground text-sm mt-1">Tailored to your needs</p>
                </div>
                <ul className="space-y-4 flex-1">
                  {enterpriseFeatures.map((feature, index) => (
                    <li key={index} className="flex items-center gap-3 text-muted-foreground">
                      <div className="w-5 h-5 rounded-full bg-primary/10 flex items-center justify-center shrink-0">
                        <Check className="h-3 w-3 text-primary" />
                      </div>
                      <span>{feature}</span>
                    </li>
                  ))}
                </ul>
                <Button
                  asChild
                  variant="outline"
                  className="w-full border-white/20 bg-transparent hover:bg-white/5 text-foreground mt-8"
                >
                  <Link href="/contact">Request Demo</Link>
                </Button>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </section>
  )
}
