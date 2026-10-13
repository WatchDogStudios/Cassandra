import type React from "react"
import type { Metadata } from "next"

export const metadata: Metadata = {
  title: "Sign Up - Infiner",
  description:
    "Create your free Infiner account. Get $10 in free credits and start building with AI inference in minutes.",
}

export default function SignupLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
