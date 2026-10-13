import type React from "react"
import type { Metadata } from "next"

export const metadata: Metadata = {
  title: "Privacy Policy - Infiner",
  description: "Infiner Privacy Policy. Learn how we collect, use, and protect your data.",
}

export default function PrivacyLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
