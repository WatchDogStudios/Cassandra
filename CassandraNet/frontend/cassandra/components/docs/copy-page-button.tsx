"use client"

import { useState } from "react"
import { Check, FileText } from "lucide-react"
import { fullDocsMarkdown } from "@/lib/docs-content"

export function CopyPageButton() {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(fullDocsMarkdown)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <button
      onClick={handleCopy}
      className="inline-flex items-center gap-2 px-3 py-1.5 text-sm text-muted-foreground hover:text-foreground bg-surface-dark border border-white/10 rounded-md transition-all hover:border-white/20"
      aria-label="Copy entire docs page as Markdown"
    >
      {copied ? (
        <>
          <Check className="w-4 h-4 text-green-500" />
          <span className="text-green-500">Copied!</span>
        </>
      ) : (
        <>
          <FileText className="w-4 h-4" />
          <span>Copy as MD</span>
        </>
      )}
    </button>
  )
}
