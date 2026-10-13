import { Button } from "@/components/ui/button"
import { Meteors } from "@/components/ui/meteors"
import { GridBackground } from "./grid-background"
import { LogoMarquee } from "./logo-marquee"
import Link from "next/link"

export function HeroSection() {
  return (
    <section className="relative min-h-[100dvh] flex flex-col bg-background overflow-hidden">
      {/* Background effects */}
      <GridBackground />
      <Meteors number={10} />

      {/* Hero content */}
      <div className="relative z-10 flex-1 flex flex-col items-center justify-center px-6 pt-24 pb-16">
        <div className="text-center max-w-4xl mx-auto">
          <h1 className="text-4xl sm:text-5xl md:text-6xl lg:text-7xl font-semibold tracking-tight text-balance">
            <span className="block text-foreground animate-slide-in-right">Infinitely scaleable</span>
            <span className="block text-muted-foreground animate-slide-up-delayed">Infinitely Secure.</span>
          </h1>

          <p className="mt-6 text-base sm:text-lg text-muted-foreground max-w-xl mx-auto text-pretty animate-slide-up-delayed-2">
            Get servers that scale with you, and security that keeps the bad guys out.
          </p>

          <div className="mt-10 flex flex-col sm:flex-row items-center justify-center gap-4 animate-slide-up-delayed-3">
            <Button size="lg" className="bg-primary hover:bg-primary/90 text-primary-foreground px-8">
              Alpha Waitlist
            </Button>
            <Link
              href="/models"
              className="inline-flex items-center justify-center h-10 rounded-md px-8 text-sm font-medium border border-border/50 bg-transparent text-foreground hover:bg-foreground/5 transition-all"
            >
              See what we can do. 
            </Link>
          </div>
        </div>
      </div>

      {/* Trusted by section */}
      <div className="relative z-10 pb-12 animate-fade-in-delayed">
        <LogoMarquee />
      </div>
    </section>
  )
}
