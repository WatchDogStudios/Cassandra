"use client"

import type React from "react"

import { useState, useEffect } from "react"
import Link from "next/link"
import Image from "next/image"
import { usePathname, useRouter } from "next/navigation"
import { Menu, X } from "lucide-react"
import { useLenis } from "./lenis-provider"

const navLinks = [
  { label: "Platform", href: "/#platform" },
  { label: "Models", href: "/models" },
  { label: "Playground", href: "/playground" },
  { label: "Pricing", href: "/#pricing" },
]

export function Navbar() {
  const [isVisible, setIsVisible] = useState(true)
  const [lastScrollY, setLastScrollY] = useState(0)
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false)
  const pathname = usePathname()
  const router = useRouter()
  const { setScrollLocked } = useLenis()

  useEffect(() => {
    const handleScroll = () => {
      const currentScrollY = window.scrollY

      if (currentScrollY < lastScrollY || currentScrollY < 50) {
        setIsVisible(true)
      } else {
        setIsVisible(false)
      }

      setLastScrollY(currentScrollY)
    }

    window.addEventListener("scroll", handleScroll, { passive: true })
    return () => window.removeEventListener("scroll", handleScroll)
  }, [lastScrollY])

  useEffect(() => {
    setScrollLocked(isMobileMenuOpen)
    return () => setScrollLocked(false)
  }, [isMobileMenuOpen, setScrollLocked])

  const handleAnchorClick = (e: React.MouseEvent<HTMLAnchorElement>, href: string) => {
    if (href.startsWith("/#")) {
      const hash = href.substring(1)

      if (pathname === "/") {
        e.preventDefault()
        const element = document.querySelector(hash)
        if (element) {
          element.scrollIntoView({ behavior: "smooth" })
        }
      } else {
        e.preventDefault()
        router.push("/")
        setTimeout(() => {
          const element = document.querySelector(hash)
          if (element) {
            element.scrollIntoView({ behavior: "smooth" })
          }
        }, 100)
      }
    }
    setIsMobileMenuOpen(false)
  }

  const handleLinkClick = (href: string) => {
    setIsMobileMenuOpen(false)
    if (!href.startsWith("/#")) {
      router.push(href)
    }
  }

  return (
    <>
      <header
        className={`fixed top-0 left-0 right-0 z-50 animate-navbar-drop backdrop-blur-md bg-background/80 border-b border-border/50 transition-transform duration-300 ease-in-out ${
          isVisible ? "translate-y-0" : "-translate-y-full"
        }`}
      >
        <div className="container mx-auto px-6 py-4">
          <nav className="flex items-center justify-between">
            {/* Logo */}
            <Link href="/" className="flex items-center">
              <Image src="/CASS_LOGO_ICON_BLACK_SMALL.png" alt="Infiner" width={50} height={38} priority />
            </Link>

            {/* Navigation Links - Desktop */}
            <div className="hidden md:flex items-center gap-8">
              {navLinks.map((link) => (
                <Link
                  key={link.label}
                  href={link.href}
                  onClick={(e) => handleAnchorClick(e, link.href)}
                  className="nav-link-hover text-sm text-muted-foreground hover:text-foreground transition-colors"
                >
                  {link.label}
                </Link>
              ))}
            </div>

            {/* CTA Buttons */}
            <div className="flex items-center gap-3">
              <Link
                href="/login"
                className="text-sm text-muted-foreground hover:text-foreground transition-colors hidden sm:block"
              >
                Login
              </Link>
              <Link
                href="/signup"
                className="group relative rounded-md px-4 py-1.5 text-sm font-medium text-foreground bg-background transition-colors hidden sm:block"
              >
                <span
                  className="absolute -inset-px rounded-md -z-10 opacity-100 group-hover:opacity-0 transition-opacity duration-[450ms] ease-in-out"
                  style={{
                    background: "linear-gradient(to bottom right, hsla(218, 100%, 65%, 1), hsla(0, 20%, 99%, 0))",
                  }}
                />
                <span
                  className="absolute -inset-px rounded-md -z-10 opacity-0 group-hover:opacity-100 transition-opacity duration-[450ms] ease-in-out"
                  style={{
                    background: "linear-gradient(to top left, hsla(218, 100%, 65%, 1), hsla(0, 20%, 99%, 0))",
                  }}
                />
                Get Started
              </Link>

              <button
                onClick={() => setIsMobileMenuOpen(!isMobileMenuOpen)}
                className="md:hidden relative z-50 p-2 text-foreground"
                aria-label={isMobileMenuOpen ? "Close menu" : "Open menu"}
              >
                <div className="relative w-6 h-6">
                  <Menu
                    className={`absolute inset-0 w-6 h-6 transition-all duration-300 ${
                      isMobileMenuOpen ? "opacity-0 rotate-90 scale-0" : "opacity-100 rotate-0 scale-100"
                    }`}
                  />
                  <X
                    className={`absolute inset-0 w-6 h-6 transition-all duration-300 ${
                      isMobileMenuOpen ? "opacity-100 rotate-0 scale-100" : "opacity-0 -rotate-90 scale-0"
                    }`}
                  />
                </div>
              </button>
            </div>
          </nav>
        </div>
      </header>

      <div
        className={`fixed inset-0 z-40 md:hidden transition-all duration-500 ease-out ${
          isMobileMenuOpen ? "pointer-events-auto" : "pointer-events-none"
        }`}
      >
        {/* Background overlay with fade */}
        <div
          className={`absolute inset-0 bg-background transition-opacity duration-500 ${
            isMobileMenuOpen ? "opacity-100" : "opacity-0"
          }`}
        />

        {/* Animated gradient background */}
        <div
          className={`absolute inset-0 transition-all duration-700 delay-100 ${
            isMobileMenuOpen ? "opacity-100" : "opacity-0"
          }`}
          style={{
            background: "radial-gradient(circle at 50% 0%, hsl(var(--primary) / 0.15) 0%, transparent 50%)",
          }}
        />

        {/* Menu content */}
        <div className="relative h-full flex flex-col justify-center items-center px-6">
          {/* Navigation links with staggered animation */}
          <nav className="flex flex-col items-center gap-2">
            {navLinks.map((link, index) => (
              <Link
                key={link.label}
                href={link.href}
                onClick={(e) => handleAnchorClick(e, link.href)}
                className={`text-4xl sm:text-5xl font-bold text-foreground hover:text-primary transition-all duration-500 ${
                  isMobileMenuOpen ? "opacity-100 translate-y-0" : "opacity-0 translate-y-8"
                }`}
                style={{
                  transitionDelay: isMobileMenuOpen ? `${150 + index * 75}ms` : "0ms",
                }}
              >
                {link.label}
              </Link>
            ))}
          </nav>

          {/* CTA buttons with delayed animation */}
          <div
            className={`mt-12 flex flex-col items-center gap-4 transition-all duration-500 ${
              isMobileMenuOpen ? "opacity-100 translate-y-0" : "opacity-0 translate-y-8"
            }`}
            style={{
              transitionDelay: isMobileMenuOpen ? "450ms" : "0ms",
            }}
          >
            <Link
              href="/login"
              onClick={() => handleLinkClick("/login")}
              className="text-lg text-muted-foreground hover:text-foreground transition-colors"
            >
              Login
            </Link>
            <Link
              href="/signup"
              onClick={() => handleLinkClick("/signup")}
              className="group relative rounded-lg px-8 py-3 text-lg font-medium text-foreground bg-background transition-colors"
            >
              <span
                className="absolute -inset-px rounded-lg -z-10 opacity-100 group-hover:opacity-0 transition-opacity duration-[450ms] ease-in-out"
                style={{
                  background: "linear-gradient(to bottom right, hsla(218, 100%, 65%, 1), hsla(0, 20%, 99%, 0))",
                }}
              />
              <span
                className="absolute -inset-px rounded-lg -z-10 opacity-0 group-hover:opacity-100 transition-opacity duration-[450ms] ease-in-out"
                style={{
                  background: "linear-gradient(to top left, hsla(218, 100%, 65%, 1), hsla(0, 20%, 99%, 0))",
                }}
              />
              Get Started
            </Link>
          </div>

          {/* Bottom decoration line with scale animation */}
          <div
            className={`absolute bottom-12 left-1/2 -translate-x-1/2 h-px bg-gradient-to-r from-transparent via-primary/50 to-transparent transition-all duration-700 ${
              isMobileMenuOpen ? "w-48 opacity-100" : "w-0 opacity-0"
            }`}
            style={{
              transitionDelay: isMobileMenuOpen ? "500ms" : "0ms",
            }}
          />
        </div>
      </div>
    </>
  )
}
