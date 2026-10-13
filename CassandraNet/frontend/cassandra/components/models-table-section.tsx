"use client"

import { useEffect, useRef, useState } from "react"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Badge } from "@/components/ui/badge"
import Link from "next/link"
import { Zap, TrendingUp, Clock } from "lucide-react"

const models = [
  {
    name: "Llama 3.1 70B",
    provider: "Meta",
    latency: "12ms",
    throughput: "2,450",
    cost: "$0.0008",
    status: "live",
  },
  {
    name: "Mixtral 8x22B",
    provider: "Mistral",
    latency: "18ms",
    throughput: "1,890",
    cost: "$0.0012",
    status: "live",
  },
  {
    name: "Claude 3.5 Sonnet",
    provider: "Anthropic",
    latency: "24ms",
    throughput: "1,650",
    cost: "$0.0030",
    status: "live",
  },
  {
    name: "GPT-4o",
    provider: "OpenAI",
    latency: "32ms",
    throughput: "1,420",
    cost: "$0.0050",
    status: "live",
  },
  {
    name: "Gemini 1.5 Pro",
    provider: "Google",
    latency: "28ms",
    throughput: "1,580",
    cost: "$0.0035",
    status: "live",
  },
  {
    name: "Command R+",
    provider: "Cohere",
    latency: "15ms",
    throughput: "2,100",
    cost: "$0.0010",
    status: "live",
  },
]

function StreamingText({
  text,
  delay = 0,
  speed = 30,
  isVisible,
}: { text: string; delay?: number; speed?: number; isVisible: boolean }) {
  const [displayedText, setDisplayedText] = useState("")
  const [isStreaming, setIsStreaming] = useState(false)

  useEffect(() => {
    if (!isVisible) return

    const startDelay = setTimeout(() => {
      setIsStreaming(true)
      let currentIndex = 0

      const interval = setInterval(() => {
        if (currentIndex <= text.length) {
          setDisplayedText(text.slice(0, currentIndex))
          currentIndex++
        } else {
          clearInterval(interval)
        }
      }, speed)

      return () => clearInterval(interval)
    }, delay)

    return () => clearTimeout(startDelay)
  }, [isVisible, text, delay, speed])

  return (
    <span className="inline-flex items-center">
      {displayedText}
      {isStreaming && displayedText.length < text.length && (
        <span className="ml-0.5 h-4 w-0.5 bg-primary animate-pulse" />
      )}
    </span>
  )
}

export function ModelsTableSection() {
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

  return (
    <section ref={sectionRef} className="relative py-24 md:py-32 bg-surface-dark overflow-hidden">
      <div
        className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-[800px] h-[150px] bg-primary/10 rounded-full blur-[100px] pointer-events-none"
        aria-hidden="true"
      />

      <div className="w-full flex justify-center px-4 relative z-10">
        <div className="flex flex-col items-center gap-12 max-w-5xl w-full">
          {/* Section header */}
          <div className="flex flex-col items-center gap-4 text-center max-w-2xl">
            <h2
              className={`text-3xl md:text-4xl lg:text-5xl font-bold text-foreground text-balance ${
                isVisible ? "animate-slide-up-section" : "opacity-0"
              }`}
            >
              Lightning-fast models at your fingertips
            </h2>
            <p
              className={`text-muted-foreground text-lg leading-relaxed ${
                isVisible ? "animate-slide-up-section-delayed" : "opacity-0"
              }`}
            >
              Access the most popular LLMs through a single API with industry-leading latency and throughput.
            </p>
          </div>

          {/* Stats row */}
          <div
            className={`grid grid-cols-3 gap-8 md:gap-16 ${
              isVisible ? "animate-slide-up-section-delayed" : "opacity-0"
            }`}
          >
            <div className="flex flex-col items-center gap-2">
              <div className="flex items-center gap-2 text-primary">
                <Zap className="h-5 w-5" />
                <span className="text-2xl md:text-3xl font-bold text-foreground">
                  <StreamingText text="12ms" delay={200} speed={60} isVisible={isVisible} />
                </span>
              </div>
              <span className="text-sm text-muted-foreground">Avg. Latency</span>
            </div>
            <div className="flex flex-col items-center gap-2">
              <div className="flex items-center gap-2 text-primary">
                <TrendingUp className="h-5 w-5" />
                <span className="text-2xl md:text-3xl font-bold text-foreground">
                  <StreamingText text="99.99%" delay={400} speed={50} isVisible={isVisible} />
                </span>
              </div>
              <span className="text-sm text-muted-foreground">Uptime</span>
            </div>
            <div className="flex flex-col items-center gap-2">
              <div className="flex items-center gap-2 text-primary">
                <Clock className="h-5 w-5" />
                <span className="text-2xl md:text-3xl font-bold text-foreground">
                  <StreamingText text="50+" delay={600} speed={80} isVisible={isVisible} />
                </span>
              </div>
              <span className="text-sm text-muted-foreground">Models</span>
            </div>
          </div>

          {/* Models table */}
          <div
            className={`w-full rounded-xl border border-white/5 bg-surface-elevated backdrop-blur-sm overflow-hidden ${
              isVisible ? "animate-slide-in-left" : "opacity-0"
            }`}
          >
            <Table>
              <TableHeader>
                <TableRow className="border-border hover:bg-transparent">
                  <TableHead className="text-muted-foreground font-medium">Model</TableHead>
                  <TableHead className="text-muted-foreground font-medium">Provider</TableHead>
                  <TableHead className="text-muted-foreground font-medium text-right">Latency</TableHead>
                  <TableHead className="text-muted-foreground font-medium text-right hidden md:table-cell">
                    Tokens/sec
                  </TableHead>
                  <TableHead className="text-muted-foreground font-medium text-right hidden sm:table-cell">
                    Cost/1K
                  </TableHead>
                  <TableHead className="text-muted-foreground font-medium text-right">Status</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {models.map((model, index) => (
                  <TableRow key={model.name} className="border-border hover:bg-primary/5 transition-colors">
                    <TableCell className="font-medium text-foreground">
                      <StreamingText text={model.name} delay={index * 150} speed={25} isVisible={isVisible} />
                    </TableCell>
                    <TableCell className="text-muted-foreground">{model.provider}</TableCell>
                    <TableCell className="text-right font-mono text-primary">
                      <StreamingText text={model.latency} delay={index * 150 + 300} speed={40} isVisible={isVisible} />
                    </TableCell>
                    <TableCell className="text-right font-mono text-foreground hidden md:table-cell">
                      <StreamingText
                        text={model.throughput}
                        delay={index * 150 + 400}
                        speed={30}
                        isVisible={isVisible}
                      />
                    </TableCell>
                    <TableCell className="text-right font-mono text-muted-foreground hidden sm:table-cell">
                      {model.cost}
                    </TableCell>
                    <TableCell className="text-right">
                      <Badge variant="outline" className="bg-green-500/10 text-green-400 border-green-500/20">
                        {model.status}
                      </Badge>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>

          <Link
            href="/models"
            className={`inline-flex items-center justify-center rounded-md text-sm font-medium h-11 px-8 border border-white/10 bg-transparent hover:bg-white/5 text-foreground transition-colors ${
              isVisible ? "animate-slide-up-section-delayed" : "opacity-0"
            }`}
          >
            Explore all models
          </Link>
        </div>
      </div>
    </section>
  )
}
