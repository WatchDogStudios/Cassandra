"use client"

import type React from "react"

import { useState } from "react"
import Link from "next/link"
import Image from "next/image"
import { Twitter, Github, Linkedin, ArrowRight } from "lucide-react"
import { ThemeSwitcher } from "@/components/theme-switcher"

const footerLinks = {
  Product: [
    { label: "Platform", href: "/#platform" },
    { label: "Models", href: "/models" }, // linked to models page
    { label: "Pricing", href: "/#pricing" }, // linked to pricing section
    { label: "Changelog", href: "#" },
  ],
  Developers: [
    { label: "Documentation", href: "/docs" }, // linked to docs page
    { label: "API Reference", href: "/docs#api-reference" }, // linked to API reference section
    { label: "Style Guide", href: "/style-guide" }, // Added Style Guide link
    { label: "Status", href: "#" },
  ],
  Company: [
    { label: "About", href: "/about" }, // Updated About link to /about page
    { label: "Blog", href: "/blog" }, // updated Blog link to point to /blog
    { label: "Careers", href: "#" },
    { label: "Contact", href: "/contact" }, // linked to contact page
  ],
  Legal: [
    { label: "Privacy", href: "/privacy" }, // linked Privacy to new page
    { label: "Terms", href: "/terms" }, // linked Terms to new page
    { label: "Security", href: "#" },
    { label: "DPA", href: "#" },
  ],
}

const socialLinks = [
  { icon: Twitter, href: "#", label: "Twitter" },
  { icon: Github, href: "#", label: "GitHub" },
  { icon: Linkedin, href: "#", label: "LinkedIn" },
]

export function Footer() {
  const [email, setEmail] = useState("")
  const [isSubmitted, setIsSubmitted] = useState(false)

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (email) {
      setIsSubmitted(true)
      setEmail("")
    }
  }

  return (
    <footer className="relative bg-background border-t border-white/5">
      {/* Subtle glow effect */}
      <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[600px] h-[1px] opacity-50 bg-gradient-to-r from-transparent via-primary to-transparent" />

      <div className="container mx-auto px-6 pt-16 pb-8">
        {/* Top section with newsletter */}
        <div className="flex flex-col lg:flex-row justify-between gap-12 pb-12 border-b border-white/5">
          {/* Newsletter signup */}
          <div className="max-w-md">
            <h3 className="text-xl font-semibold text-foreground mb-2">Stay ahead of the curve</h3>
            <p className="text-muted-foreground text-sm mb-6">
              Get the latest updates on new models, features, and AI insights delivered to your inbox.
            </p>

            {isSubmitted ? (
              <div className="flex items-center gap-2 text-primary">
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                <span className="text-sm">Thanks for subscribing!</span>
              </div>
            ) : (
              <form onSubmit={handleSubmit} className="flex gap-2">
                <div className="relative flex-1">
                  <input
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="Enter your email"
                    className="w-full px-4 py-2.5 bg-white/5 border border-white/10 rounded-lg text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/50 transition-colors"
                    required
                  />
                </div>
                <button
                  type="submit"
                  className="group flex items-center gap-2 px-5 py-2.5 bg-primary hover:bg-primary/90 text-white text-sm font-medium rounded-lg transition-colors"
                >
                  Subscribe
                  <ArrowRight className="w-4 h-4 group-hover:translate-x-0.5 transition-transform" />
                </button>
              </form>
            )}
          </div>

          {/* Social links */}
          <div className="flex items-start gap-6">
            <ThemeSwitcher />
            <div className="flex items-start gap-4">
              {socialLinks.map((social) => (
                <Link
                  key={social.label}
                  href={social.href}
                  className="p-2.5 rounded-lg bg-white/5 border border-white/10 text-muted-foreground hover:text-foreground hover:border-white/20 transition-colors"
                  aria-label={social.label}
                >
                  <social.icon className="w-5 h-5" />
                </Link>
              ))}
            </div>
          </div>
        </div>

        {/* Links grid */}
        <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-8 py-12">
          {/* Logo and tagline */}
          <div className="col-span-2 md:col-span-4 lg:col-span-1 mb-4 lg:mb-0">
            <Link href="/" className="inline-block mb-4">
              <Image src="/CASS_LOGO_ICON_BLACK_SMALL.png" alt="Cassandra" width={120} height={28} />
            </Link>
            <p className="text-sm text-muted-foreground max-w-50">
              Fast, Scalable, and Effortless.
            </p>
          </div>

          {/* Link columns */}
          {Object.entries(footerLinks).map(([category, links]) => (
            <div key={category}>
              <h4 className="text-sm font-medium text-foreground mb-4">{category}</h4>
              <ul className="space-y-3">
                {links.map((link) => (
                  <li key={link.label}>
                    <Link
                      href={link.href}
                      className="text-sm text-muted-foreground hover:text-foreground transition-colors"
                    >
                      {link.label}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        {/* Bottom bar */}
        <div className="flex flex-col sm:flex-row justify-between items-center gap-4 pt-8 border-t border-white/5">
          <p className="text-xs text-muted-foreground">© {new Date().getFullYear()} WD Studios Corp. All rights reserved. Cassandra, CassandraNet, and related trademarks are the property of WD Studios Corp.</p>
          <div className="flex items-center gap-2">
            <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
              All systems operational
            </span>
          </div>
        </div>
      </div>
    </footer>
  )
}
