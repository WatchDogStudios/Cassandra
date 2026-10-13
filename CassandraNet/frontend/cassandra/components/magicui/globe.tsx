"use client"

import { useCallback, useEffect, useRef, useState } from "react"
import { useTheme } from "next-themes"
import { cn } from "@/lib/utils"

const GLOBE_BASE_CONFIG = {
  width: 800,
  height: 800,
  onRender: () => {},
  devicePixelRatio: 2,
  phi: 0,
  theta: 0.3,
  diffuse: 0.4,
  mapSamples: 16000,
  mapBrightness: 1.2,
  markers: [
    { location: [14.5995, 120.9842], size: 0.03 },
    { location: [19.076, 72.8777], size: 0.1 },
    { location: [23.8103, 90.4125], size: 0.05 },
    { location: [30.0444, 31.2357], size: 0.07 },
    { location: [39.9042, 116.4074], size: 0.08 },
    { location: [-23.5505, -46.6333], size: 0.1 },
    { location: [19.4326, -99.1332], size: 0.1 },
    { location: [40.7128, -74.006], size: 0.1 },
    { location: [34.6937, 135.5022], size: 0.05 },
    { location: [41.0082, 28.9784], size: 0.06 },
  ],
}

const DARK_THEME_CONFIG = {
  dark: 1,
  baseColor: [0.13, 0.24, 0.55] as [number, number, number],
  markerColor: [0.34, 0.37, 0.87] as [number, number, number],
  glowColor: [0.13, 0.24, 0.55] as [number, number, number],
}

const LIGHT_THEME_CONFIG = {
  dark: 0,
  baseColor: [0.6, 0.7, 0.9] as [number, number, number],
  markerColor: [0.13, 0.37, 0.87] as [number, number, number],
  glowColor: [0.6, 0.7, 0.9] as [number, number, number],
}

export function Globe({
  className,
}: {
  className?: string
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const pointerInteracting = useRef<number | null>(null)
  const pointerInteractionMovement = useRef(0)
  const [r, setR] = useState(0)
  const [isLoaded, setIsLoaded] = useState(false)
  const phiRef = useRef(0)
  const widthRef = useRef(0)
  const globeRef = useRef<{ destroy: () => void } | null>(null)

  const { resolvedTheme } = useTheme()
  const isDark = resolvedTheme === "dark"

  const updatePointerInteraction = (value: number | null) => {
    pointerInteracting.current = value
    if (canvasRef.current) {
      canvasRef.current.style.cursor = value ? "grabbing" : "grab"
    }
  }

  const updateMovement = (clientX: number) => {
    if (pointerInteracting.current !== null) {
      const delta = clientX - pointerInteracting.current
      pointerInteractionMovement.current = delta
      setR(delta / 200)
    }
  }

  const onRender = useCallback(
    (state: Record<string, unknown>) => {
      if (!pointerInteracting.current) phiRef.current += 0.005
      state.phi = phiRef.current + r
      state.width = widthRef.current * 2
      state.height = widthRef.current * 2
    },
    [r],
  )

  const onResize = () => {
    if (canvasRef.current) {
      widthRef.current = canvasRef.current.offsetWidth
    }
  }

  useEffect(() => {
    const initGlobe = async () => {
      try {
        // Destroy existing globe if any
        if (globeRef.current) {
          globeRef.current.destroy()
          globeRef.current = null
        }

        const createGlobe = (await import("cobe")).default

        if (!canvasRef.current) return

        window.addEventListener("resize", onResize)
        onResize()

        const themeConfig = isDark ? DARK_THEME_CONFIG : LIGHT_THEME_CONFIG

        globeRef.current = createGlobe(canvasRef.current, {
          ...GLOBE_BASE_CONFIG,
          ...themeConfig,
          width: widthRef.current * 2,
          height: widthRef.current * 2,
          onRender,
        })

        setIsLoaded(true)

        setTimeout(() => {
          if (canvasRef.current) {
            canvasRef.current.style.opacity = "1"
          }
        }, 100)
      } catch (error) {
        console.error("Failed to load globe:", error)
        setIsLoaded(true)
      }
    }

    initGlobe()

    return () => {
      globeRef.current?.destroy()
      window.removeEventListener("resize", onResize)
    }
  }, [onRender, isDark]) // Re-run when theme changes

  return (
    <div className={cn("absolute inset-0 mx-auto aspect-[1/1] w-full max-w-[600px]", className)}>
      {!isLoaded && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="size-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
        </div>
      )}
      <canvas
        className="size-full opacity-0 transition-opacity duration-500 [contain:layout_paint_size]"
        ref={canvasRef}
        onPointerDown={(e) => updatePointerInteraction(e.clientX - pointerInteractionMovement.current)}
        onPointerUp={() => updatePointerInteraction(null)}
        onPointerOut={() => updatePointerInteraction(null)}
        onMouseMove={(e) => updateMovement(e.clientX)}
        onTouchMove={(e) => e.touches[0] && updateMovement(e.touches[0].clientX)}
      />
    </div>
  )
}
