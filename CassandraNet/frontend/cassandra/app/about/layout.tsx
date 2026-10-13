import type React from "react"
import type { Metadata } from "next"

export const metadata: Metadata = {
  title: "About Us - Infiner",
  description:
    "Meet the team behind Infiner. Learn about our mission to make AI inference infinitely scalable and affordable for every developer.",
  openGraph: {
    title: "About Us - Infiner",
    description:
      "Meet the team behind Infiner. Learn about our mission to make AI inference infinitely scalable and affordable for every developer.",
  },
}

export default function AboutLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return children
}
