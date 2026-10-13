import Link from "next/link"
import { Button } from "@/components/ui/button"
import { Navbar } from "@/components/navbar"
import { Footer } from "@/components/footer"
import { Home, Search } from "lucide-react"

export default function NotFound() {
  return (
    <main className="min-h-screen bg-background flex flex-col">
      <Navbar />

      <div className="flex-1 flex items-center justify-center relative overflow-hidden py-24 md:py-32">
        {/* Background grid */}
        <div className="absolute inset-0 overflow-hidden">
          <div
            className="absolute inset-0 opacity-30"
            style={{
              backgroundImage: `
                linear-gradient(var(--grid-color) 1px, transparent 1px),
                linear-gradient(90deg, var(--grid-color) 1px, transparent 1px)
              `,
              backgroundSize: "60px 60px",
            }}
          />
        </div>

        {/* Content */}
        <div className="relative z-10 text-center px-4 max-w-2xl mx-auto">
          <div className="relative mb-8">
            <h1 className="text-[12rem] md:text-[16rem] font-bold leading-none tracking-tighter select-none">
              <span className="bg-gradient-to-b from-primary via-primary/80 to-primary/20 bg-clip-text text-transparent">
                404
              </span>
            </h1>
          </div>

          {/* Message - removed animation classes */}
          <div className="space-y-4">
            <h2 className="text-2xl md:text-3xl font-semibold text-foreground">Page not found</h2>
            <p className="text-muted-foreground text-lg max-w-md mx-auto">
              The page you&apos;re looking for doesn&apos;t exist or has been moved to another location.
            </p>
          </div>

          {/* Action buttons - removed animation classes */}
          <div className="flex flex-col sm:flex-row items-center justify-center gap-4 mt-10">
            <Button asChild size="lg" className="group">
              <Link href="/">
                <Home className="mr-2 h-4 w-4 transition-transform group-hover:-translate-y-0.5" />
                Back to Home
              </Link>
            </Button>
            <Button asChild variant="outline" size="lg" className="group bg-transparent">
              <Link href="/docs">
                <Search className="mr-2 h-4 w-4 transition-transform group-hover:scale-110" />
                Browse Docs
              </Link>
            </Button>
          </div>

          {/* Quick links - removed animation classes */}
          <div className="mt-16">
            <p className="text-sm text-muted-foreground mb-4">Or check out these pages:</p>
            <div className="flex flex-wrap items-center justify-center gap-3">
              {[
                { label: "Models", href: "/models" },
                { label: "Playground", href: "/playground" },
                { label: "Blog", href: "/blog" },
                { label: "About", href: "/about" },
                { label: "Contact", href: "/contact" },
              ].map((link) => (
                <Link
                  key={link.href}
                  href={link.href}
                  className="px-4 py-2 text-sm rounded-full border border-border bg-card/50 text-muted-foreground hover:text-foreground hover:border-primary/50 hover:bg-primary/5 transition-all duration-300"
                >
                  {link.label}
                </Link>
              ))}
            </div>
          </div>
        </div>
      </div>

      <Footer />
    </main>
  )
}
