"use client"
import { cn } from "@/lib/utils"
import { useEffect, useState } from "react"

export const Meteors = ({
  number,
  className,
}: {
  number?: number
  className?: string
}) => {
  const [meteorStyles, setMeteorStyles] = useState<
    Array<{ left: string; top: string; animationDelay: string; animationDuration: string }>
  >([])

  useEffect(() => {
    const styles = Array.from({ length: number || 10 }, () => ({
      left: Math.floor(Math.random() * 120) + "%",
      top: Math.floor(Math.random() * 20) - 30 + "%",
      animationDelay: Math.random() * 12 + "s",
      animationDuration: Math.floor(Math.random() * 4 + 3) + "s",
    }))
    setMeteorStyles(styles)
  }, [number])

  return (
    <div className={cn("absolute inset-0 overflow-hidden pointer-events-none", className)}>
      {meteorStyles.map((style, idx) => (
        <span
          key={"meteor" + idx}
          className={cn(
            "animate-meteor-effect absolute h-0.5 w-0.5 rounded-full bg-white shadow-[0_0_3px_1px_rgba(255,255,255,0.2)]",
            "before:absolute before:bottom-full before:left-1/2 before:-translate-x-1/2 before:h-[120px] before:w-[1px] before:bg-gradient-to-t before:from-white/80 before:via-blue-400/30 before:to-transparent before:content-['']",
          )}
          style={{
            top: style.top,
            left: style.left,
            animationDelay: style.animationDelay,
            animationDuration: style.animationDuration,
          }}
        />
      ))}
    </div>
  )
}
