import type React from "react"
import type { Metadata } from "next"

export const metadata: Metadata = {
  title: "Contact Us - Infiner",
  description:
    "Get in touch with our team. Schedule a demo, discuss enterprise solutions, or learn how Infiner can power your AI applications.",
  openGraph: {
    title: "Contact Us - Infiner",
    description: "Get in touch with our team. Schedule a demo or discuss enterprise solutions.",
  },
}

export default function ContactLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
