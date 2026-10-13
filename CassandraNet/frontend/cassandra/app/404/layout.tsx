import type React from "react"
import type { Metadata } from "next"

export const metadata: Metadata = {
  title: "404 - Page Not Found | Infiner",
  description: "The page you're looking for doesn't exist or has been moved.",
}

export default function NotFoundLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
