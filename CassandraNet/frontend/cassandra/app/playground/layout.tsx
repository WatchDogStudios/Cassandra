import type React from "react"
import type { Metadata } from "next"

export const metadata: Metadata = {
  title: "API Playground - Infiner",
  description:
    "Test AI models in real-time with our interactive playground. No API key required. Compare responses across different models.",
  openGraph: {
    title: "API Playground - Infiner",
    description: "Test AI models in real-time with our interactive playground. No API key required.",
  },
}

export default function PlaygroundLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
