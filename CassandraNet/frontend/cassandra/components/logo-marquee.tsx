"use client"

import Image from "next/image"
import { useTheme } from "next-themes"
import { useEffect, useState } from "react"

const logoCount = 8

export function LogoMarquee() {
  const { resolvedTheme } = useTheme()
  const [mounted, setMounted] = useState(false)

  useEffect(() => {
    setMounted(true)
  }, [])

  // Determine filter based on theme - invert for dark mode to make logos white
  const logoFilter =
    mounted && resolvedTheme === "dark" ? "grayscale(100%) brightness(0) invert(1)" : "grayscale(100%) brightness(0)"

  return (
    <div className="relative w-full max-w-5xl mx-auto px-4 md:px-6 flex flex-col items-center gap-6 md:gap-10 overflow-hidden">
      <p className="text-muted-foreground/60 text-sm md:text-lg text-center font-medium tracking-wide">
        Trusted by top innovative teams
      </p>

      <div
        className="relative overflow-hidden w-full"
        style={{
          maskImage: "linear-gradient(to right, transparent, black 15%, black 85%, transparent)",
          WebkitMaskImage: "linear-gradient(to right, transparent, black 15%, black 85%, transparent)",
        }}
      >
        <div className="flex gap-8 md:gap-20 animate-marquee">
          {Array.from({ length: logoCount * 2 }).map((_, index) => (
            <div key={index} className="shrink-0">
              <Image
                src="/placeholder-logo.svg"
                alt="Company logo"
                width={120}
                height={40}
                priority
                className="h-8 md:h-10 w-auto opacity-70"
                style={{ filter: logoFilter }}
              />
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}
