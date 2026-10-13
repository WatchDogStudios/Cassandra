import type React from "react"
import type { Metadata } from "next"

export const metadata: Metadata = {
  title: "Documentation - Infiner",
  description:
    "Complete API documentation for Infiner. Learn how to integrate our inference platform with quickstart guides, code examples, and API reference.",
  openGraph: {
    title: "Documentation - Infiner",
    description: "Complete API documentation with quickstart guides, code examples, and API reference.",
  },
}

export default function DocsLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
