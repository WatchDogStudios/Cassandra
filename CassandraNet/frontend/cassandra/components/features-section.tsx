"use client"
import React, { useRef, useEffect, useState } from "react"
import { cn } from "@/lib/utils"
import { Globe } from "@/components/magicui/globe"
import { AnimatedBeam } from "@/components/magicui/animated-beam"
import { Card, CardContent, CardHeader } from "@/components/ui/card"
import { Cpu, Zap, Server, Brain, Layers, Cloud, Shield, BarChart3, Lock, Activity } from "lucide-react"

interface FeaturesSectionProps {
  className?: string
}

const IconCard = React.forwardRef<HTMLDivElement, { icon: React.ReactNode; className?: string }>(
  ({ icon, className }, ref) => {
    return (
      <div
        ref={ref}
        className={cn(
          "relative z-10 flex size-14 items-center justify-center rounded-xl border border-border bg-card",
          className,
        )}
      >
        {icon}
      </div>
    )
  },
)
IconCard.displayName = "IconCard"

export function FeaturesSection({ className }: FeaturesSectionProps) {
  const [headerVisible, setHeaderVisible] = useState(false)
  const [topRowVisible, setTopRowVisible] = useState(false)
  const [bottomRowVisible, setBottomRowVisible] = useState(false)
  const [cardVisibility, setCardVisibility] = useState<boolean[]>([false, false, false, false, false, false])
  const [isMobile, setIsMobile] = useState(false)

  const headerRef = useRef<HTMLDivElement>(null)
  const topRowRef = useRef<HTMLDivElement>(null)
  const bottomRowRef = useRef<HTMLDivElement>(null)
  const cardRefs = useRef<(HTMLDivElement | null)[]>([])

  useEffect(() => {
    const checkMobile = () => setIsMobile(window.innerWidth < 768)
    checkMobile()
    window.addEventListener("resize", checkMobile)
    return () => window.removeEventListener("resize", checkMobile)
  }, [])

  useEffect(() => {
    const headerObserver = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setHeaderVisible(true)
          headerObserver.disconnect()
        }
      },
      { threshold: 0.5 },
    )

    const topRowObserver = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setTopRowVisible(true)
          topRowObserver.disconnect()
        }
      },
      { threshold: 0.2 },
    )

    const bottomRowObserver = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setBottomRowVisible(true)
          bottomRowObserver.disconnect()
        }
      },
      { threshold: 0.3 },
    )

    if (headerRef.current) headerObserver.observe(headerRef.current)
    if (topRowRef.current) topRowObserver.observe(topRowRef.current)
    if (bottomRowRef.current) bottomRowObserver.observe(bottomRowRef.current)

    return () => {
      headerObserver.disconnect()
      topRowObserver.disconnect()
      bottomRowObserver.disconnect()
    }
  }, [])

  useEffect(() => {
    if (!isMobile) return

    const observers: IntersectionObserver[] = []

    cardRefs.current.forEach((ref, index) => {
      if (ref) {
        const observer = new IntersectionObserver(
          ([entry]) => {
            if (entry.isIntersecting) {
              setCardVisibility((prev) => {
                const newState = [...prev]
                newState[index] = true
                return newState
              })
              observer.disconnect()
            }
          },
          { threshold: 0.2 },
        )
        observer.observe(ref)
        observers.push(observer)
      }
    })

    return () => observers.forEach((obs) => obs.disconnect())
  }, [isMobile])

  const containerRef1 = useRef<HTMLDivElement>(null)
  const containerRef2 = useRef<HTMLDivElement>(null)
  const containerRef3 = useRef<HTMLDivElement>(null)
  const div1Ref = useRef<HTMLDivElement>(null)
  const div2Ref = useRef<HTMLDivElement>(null)
  const div3Ref = useRef<HTMLDivElement>(null)
  const div4Ref = useRef<HTMLDivElement>(null)
  const div5Ref = useRef<HTMLDivElement>(null)
  const div6Ref = useRef<HTMLDivElement>(null)
  const div7Ref = useRef<HTMLDivElement>(null)
  const div8Ref = useRef<HTMLDivElement>(null)
  const div9Ref = useRef<HTMLDivElement>(null)

  const getCardAnimation = (index: number, isTopRow: boolean) => {
    if (isMobile) {
      return cardVisibility[index] ? `animate-slide-in-right-mobile-${index + 1}` : "opacity-0"
    }
    // Desktop: use row-based animations
    if (isTopRow) {
      return topRowVisible ? "animate-slide-in-left" : "opacity-0"
    }
    return bottomRowVisible ? "animate-slide-in-right-delayed" : "opacity-0"
  }

  return (
    <section id="platform" className={cn("py-20 md:py-32", className)}>
      <div className="w-full flex justify-center px-4 md:px-6">
        <div className="flex max-w-5xl flex-col items-center gap-5">
          <div ref={headerRef} className="text-center max-w-2xl mb-8">
            <h2
              className={cn(
                "text-3xl md:text-4xl lg:text-5xl font-bold tracking-tight text-foreground text-balance opacity-0",
                headerVisible && "animate-slide-up-section",
              )}
            >
              Built for scale, optimized for speed
            </h2>
            <p
              className={cn(
                "mt-4 text-base md:text-lg text-muted-foreground text-balance opacity-0",
                headerVisible && "animate-slide-up-section-delayed",
              )}
            >
              Deploy anything to your servers, from small and large databases, to full game servers, Cassandra has your back.
            </p>
          </div>

          <div
            ref={topRowRef}
            className={cn(
              "grid w-full grid-cols-1 gap-5 md:grid-cols-3",
              !isMobile && "opacity-0",
              !isMobile && topRowVisible && "animate-slide-in-left",
            )}
          >
            {/* 1st Card - Model Orchestration */}
            <Card
              ref={(el) => {
                cardRefs.current[0] = el
              }}
              className={cn(
                "relative h-96 w-full rounded-3xl border border-border bg-card",
                isMobile && getCardAnimation(0, true),
              )}
            >
              <CardHeader className="text-center">
                <h3 className="text-xl md:text-2xl font-semibold tracking-tight text-foreground">
                  Server Orchestration
                </h3>
                <p className="mt-2 text-sm text-muted-foreground">
                  Route requests to the optimal server based on latency, cost, and capability. Seamlessly switch between
                  agents without code changes.
                </p>
              </CardHeader>
              <CardContent ref={containerRef1} className="relative ml-5">
                <IconCard ref={div1Ref} icon={<Brain className="size-5 text-primary" />} className="mb-3" />
                <IconCard ref={div2Ref} icon={<Cpu className="size-5 text-primary" />} className="translate-x-32" />
                <IconCard ref={div3Ref} icon={<Layers className="size-5 text-primary" />} className="mt-3" />
                <IconCard
                  ref={div5Ref}
                  icon={<Zap className="size-5 text-primary" />}
                  className="absolute top-1/2 right-12 -translate-y-1/2"
                />
                <div
                  ref={div4Ref}
                  className="absolute top-1/2 left-1/2 z-50 h-4 w-4 -translate-y-1/2 rounded-full border border-primary bg-primary/20"
                />
                <AnimatedBeam
                  duration={3}
                  containerRef={containerRef1}
                  fromRef={div1Ref}
                  curvature={40}
                  toRef={div4Ref}
                  delay={0}
                />
                <AnimatedBeam duration={3} containerRef={containerRef1} fromRef={div2Ref} toRef={div4Ref} delay={0.5} />
                <AnimatedBeam
                  duration={3}
                  containerRef={containerRef1}
                  fromRef={div3Ref}
                  curvature={-40}
                  toRef={div4Ref}
                  delay={1}
                />
                <AnimatedBeam duration={3} containerRef={containerRef1} fromRef={div4Ref} toRef={div5Ref} delay={1.5} />
              </CardContent>
            </Card>

            {/* 2nd Card - Auto Scaling */}
            <Card
              ref={(el) => {
                cardRefs.current[1] = el
              }}
              className={cn(
                "h-96 w-full rounded-3xl border border-border bg-card",
                isMobile && getCardAnimation(1, true),
              )}
            >
              <CardHeader className="text-center">
                <h3 className="text-xl md:text-2xl font-semibold tracking-tight text-foreground">Auto Scaling</h3>
                <p className="text-sm text-muted-foreground">
                  Scale from zero to millions of requests automatically with intelligent load balancing.
                </p>
              </CardHeader>
              <CardContent
                ref={containerRef2}
                className="relative flex h-40 flex-col items-center justify-between py-4"
              >
                <IconCard ref={div6Ref} icon={<Server className="size-5 text-primary" />} className="mb-3" />
                <IconCard ref={div7Ref} icon={<Cloud className="size-5 text-primary" />} className="mt-3" />
                <AnimatedBeam
                  duration={3}
                  containerRef={containerRef2}
                  fromRef={div6Ref}
                  direction="vertical"
                  curvature={40}
                  toRef={div7Ref}
                />
              </CardContent>
            </Card>

            {/* 3rd Card - Real-time Analytics */}
            <Card
              ref={(el) => {
                cardRefs.current[2] = el
              }}
              className={cn(
                "relative h-96 w-full overflow-hidden rounded-3xl border border-border bg-card",
                isMobile && getCardAnimation(2, true),
              )}
            >
              <CardHeader className="text-center">
                <h3 className="text-xl md:text-2xl font-semibold tracking-tight text-foreground">
                  Real-time Analytics
                </h3>
                <p className="text-sm text-muted-foreground">
                  Monitor Server metrics, costs, and performance all in your portal.
                </p>
              </CardHeader>
              <CardContent className="flex flex-col items-center justify-center gap-3 pt-4">
                <div className="flex w-full items-end justify-center gap-2 px-4">
                  {[40, 65, 45, 80, 55, 90, 70, 85, 60, 75].map((height, i) => (
                    <div
                      key={i}
                      className="w-4 rounded-t bg-primary/60 transition-all duration-500"
                      style={{
                        height: `${height}px`,
                        animationDelay: `${i * 100}ms`,
                        animation: "pulse 2s ease-in-out infinite",
                      }}
                    />
                  ))}
                </div>
                <div className="mt-4 flex items-center gap-2">
                  <Activity className="size-5 text-primary" />
                  <span className="text-2xl font-bold text-foreground">99.9%</span>
                  <span className="text-sm text-muted-foreground">uptime</span>
                </div>
              </CardContent>
            </Card>
          </div>

          <div
            ref={bottomRowRef}
            className={cn(
              "grid w-full grid-cols-1 gap-5 md:grid-cols-3",
              !isMobile && "opacity-0",
              !isMobile && bottomRowVisible && "animate-slide-in-right-delayed",
            )}
          >
            {/* 4th Card - Lightning Fast */}
            <Card
              ref={(el) => {
                cardRefs.current[3] = el
              }}
              className={cn(
                "relative flex h-96 w-full flex-col rounded-3xl border border-border bg-card",
                isMobile && getCardAnimation(3, false),
              )}
            >
              <CardContent className="flex items-center justify-center pt-8">
                <div className="relative flex size-32 items-center justify-center">
                  <div className="absolute inset-0 rounded-full bg-primary/10 animate-pulse" />
                  <div className="absolute inset-4 rounded-full bg-primary/20" />
                  <Zap className="size-12 text-primary" />
                </div>
              </CardContent>
              <CardHeader className="mt-auto text-center">
                <h3 className="text-xl md:text-2xl font-semibold tracking-tight text-foreground">
                  Lightning Fast
                </h3>
                <p className="text-sm text-muted-foreground">
                  Sub-100ms latency with deployments across 200+ global locations.
                </p>
              </CardHeader>
            </Card>

            {/* 5th Card - Global Infrastructure */}
            <Card
              ref={(el) => {
                cardRefs.current[4] = el
              }}
              className={cn(
                "h-96 w-full overflow-hidden rounded-3xl border border-border bg-card",
                isMobile && getCardAnimation(4, false),
              )}
            >
              <CardHeader className="text-center">
                <h3 className="text-xl md:text-2xl font-semibold tracking-tight text-foreground">
                  Global Infrastructure
                </h3>
                <p className="text-sm text-muted-foreground">
                  Deploy servers closest to your users. Automatic failover ensures 99.99% uptime across all
                  regions.
                </p>
              </CardHeader>
              <CardContent className="relative h-48">
                <Globe className="top-0" />
              </CardContent>
            </Card>

            {/* 6th Card - Enterprise Security */}
            <Card
              ref={(el) => {
                cardRefs.current[5] = el
              }}
              className={cn(
                "relative h-96 w-full overflow-hidden rounded-3xl border border-border bg-card",
                isMobile && getCardAnimation(5, false),
              )}
            >
              <CardHeader className="text-center">
                <h3 className="text-xl md:text-2xl font-semibold tracking-tight text-foreground">
                  Enterprise Security
                </h3>
                <p className="text-sm text-muted-foreground">
                  SOC 2 compliant with end-to-end encryption. Your data never leaves your VPC.
                </p>
              </CardHeader>
              <CardContent ref={containerRef3} className="relative flex h-40 items-center justify-center">
                <IconCard ref={div8Ref} icon={<Lock className="size-5 text-primary" />} className="absolute left-8" />
                <div className="relative flex size-20 items-center justify-center">
                  <div className="absolute inset-0 rounded-full border-2 border-primary/30 animate-[spin_8s_linear_infinite]" />
                  <div className="absolute inset-2 rounded-full border-2 border-dashed border-primary/50 animate-[spin_6s_linear_infinite_reverse]" />
                  <Shield className="size-8 text-primary" />
                </div>
                <IconCard
                  ref={div9Ref}
                  icon={<BarChart3 className="size-5 text-primary" />}
                  className="absolute right-8"
                />
                <AnimatedBeam
                  duration={3}
                  containerRef={containerRef3}
                  fromRef={div8Ref}
                  toRef={div9Ref}
                  curvature={-30}
                />
                <AnimatedBeam
                  duration={3}
                  containerRef={containerRef3}
                  fromRef={div8Ref}
                  toRef={div9Ref}
                  curvature={30}
                  reverse
                />
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </section>
  )
}
