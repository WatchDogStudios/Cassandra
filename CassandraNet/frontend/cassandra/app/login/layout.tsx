import type React from "react"
import type { Metadata } from "next"

export const metadata: Metadata = {
  title: "Login - Infiner",
  description: "Sign in to your Infiner account to access the dashboard, manage API keys, and monitor usage.",
}

export default function LoginLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
