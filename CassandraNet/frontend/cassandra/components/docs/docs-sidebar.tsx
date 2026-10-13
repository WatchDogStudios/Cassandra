"use client"

import type React from "react"

import { Book, Code, Zap, Shield } from "lucide-react"
import type { DocsSectionId } from "@/lib/docs-content"

interface SidebarItem {
  title: string
  href: string
  id: DocsSectionId
}

interface SidebarSection {
  title: string
  icon: React.ComponentType<{ className?: string }>
  items: SidebarItem[]
}

export const sidebarItems: SidebarSection[] = [
  {
    title: "Getting Started",
    icon: Book,
    items: [
      { title: "Introduction", href: "#introduction", id: "introduction" },
      { title: "Quickstart", href: "#quickstart", id: "quickstart" },
      { title: "Authentication", href: "#authentication", id: "authentication" },
    ],
  },
  {
    title: "API Reference",
    icon: Code,
    items: [
      { title: "Chat Completions", href: "#chat-completions", id: "chat-completions" },
      { title: "Embeddings", href: "#embeddings", id: "embeddings" },
      { title: "Models", href: "#models", id: "models" },
    ],
  },
  {
    title: "Features",
    icon: Zap,
    items: [
      { title: "Model Routing", href: "#model-routing", id: "model-routing" },
      { title: "Streaming", href: "#streaming", id: "streaming" },
      { title: "Function Calling", href: "#function-calling", id: "function-calling" },
    ],
  },
  {
    title: "Security",
    icon: Shield,
    items: [
      { title: "API Keys", href: "#api-keys", id: "api-keys" },
      { title: "Rate Limits", href: "#rate-limits", id: "rate-limits" },
    ],
  },
]

interface DocsSidebarProps {
  activeSection: DocsSectionId
  onSectionChange: (section: DocsSectionId) => void
}

export function DocsSidebar({ activeSection, onSectionChange }: DocsSidebarProps) {
  return (
    <aside className="hidden lg:block fixed left-0 top-16 bottom-0 w-64 border-r border-white/10 overflow-y-auto p-6">
      <nav className="space-y-6">
        {sidebarItems.map((section) => (
          <div key={section.title}>
            <div className="flex items-center gap-2 text-sm font-semibold text-foreground mb-3">
              <section.icon className="w-4 h-4 text-primary" />
              {section.title}
            </div>
            <ul className="space-y-1 ml-6">
              {section.items.map((item) => (
                <li key={item.href}>
                  <a
                    href={item.href}
                    onClick={() => onSectionChange(item.id)}
                    className={`block py-1.5 text-sm transition-colors ${
                      activeSection === item.id
                        ? "text-primary font-medium"
                        : "text-muted-foreground hover:text-foreground"
                    }`}
                  >
                    {item.title}
                  </a>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </nav>
    </aside>
  )
}
