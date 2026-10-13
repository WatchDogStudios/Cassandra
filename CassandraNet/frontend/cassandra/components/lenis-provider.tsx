"use client"

import type React from "react"
import { createContext, useContext, useEffect, useState, useCallback } from "react"
import { usePathname } from "next/navigation"
import Lenis from "lenis"

interface LenisContextType {
  lenis: Lenis | null
  setScrollLocked: (locked: boolean) => void
}

const LenisContext = createContext<LenisContextType>({ lenis: null, setScrollLocked: () => {} })

export function useLenis() {
  return useContext(LenisContext)
}

export function LenisProvider({ children }: { children: React.ReactNode }) {
  const [lenis, setLenisInstance] = useState<Lenis | null>(null)
  const [scrollLocked, setScrollLockedState] = useState(false)
  const pathname = usePathname()

  const setScrollLocked = useCallback((locked: boolean) => {
    setScrollLockedState(locked)
  }, [])

  useEffect(() => {
    const resizeObserverError = window.onerror
    window.onerror = (message, ...args) => {
      if (typeof message === "string" && message.includes("ResizeObserver loop")) {
        return true
      }
      return resizeObserverError ? resizeObserverError(message, ...args) : false
    }

    const lenisInstance = new Lenis({
      duration: 1.2,
      easing: (t) => Math.min(1, 1.001 - Math.pow(2, -10 * t)),
      orientation: "vertical",
      gestureOrientation: "vertical",
      smoothWheel: true,
    })

    setLenisInstance(lenisInstance)

    function raf(time: number) {
      lenisInstance.raf(time)
      requestAnimationFrame(raf)
    }

    requestAnimationFrame(raf)

    return () => {
      lenisInstance.destroy()
      window.onerror = resizeObserverError
    }
  }, [])

  useEffect(() => {
    if (lenis) {
      // Reset immediately
      lenis.scrollTo(0, { immediate: true })
      // Also use native scroll as fallback
      window.scrollTo(0, 0)
      // And after a brief delay to catch any async content
      const timeout = setTimeout(() => {
        lenis.scrollTo(0, { immediate: true })
        window.scrollTo(0, 0)
      }, 50)
      return () => clearTimeout(timeout)
    }
  }, [pathname, lenis])

  useEffect(() => {
    if (scrollLocked) {
      lenis?.stop()
      document.body.style.overflow = "hidden"
      document.body.style.position = "fixed"
      document.body.style.inset = "0"
      document.body.style.width = "100%"
    } else {
      lenis?.start()
      document.body.style.overflow = ""
      document.body.style.position = ""
      document.body.style.inset = ""
      document.body.style.width = ""
    }
  }, [scrollLocked, lenis])

  return <LenisContext.Provider value={{ lenis, setScrollLocked }}>{children}</LenisContext.Provider>
}
