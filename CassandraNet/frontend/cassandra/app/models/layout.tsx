import type React from "react"
import type { Metadata } from "next"

export const metadata: Metadata = {
  title: "Models - Infiner",
  description:
    "Browse 40+ AI models from OpenAI, Anthropic, Meta, Google, and more. Compare pricing, context windows, and capabilities.",
  openGraph: {
    title: "Models - Infiner",
    description: "Browse 40+ AI models from top providers. Compare pricing, context windows, and capabilities.",
  },
}

export default function ModelsLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
