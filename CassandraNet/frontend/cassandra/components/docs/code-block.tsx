"use client"

import { useState } from "react"
import { Check, Copy } from "lucide-react"
import { SyntaxHighlighter } from "@/lib/syntax-highlighter"

interface CodeBlockProps {
  code: string
  language?: string
}

export function CodeBlock({ code, language = "javascript" }: CodeBlockProps) {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    await navigator.clipboard.writeText(code)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const isHighlightable = ["javascript", "typescript", "js", "ts", "jsx", "tsx"].includes(language)

  return (
    <div className="relative group">
      <pre className="bg-surface-dark border border-white/10 rounded-lg p-4 overflow-x-auto text-sm">
        {isHighlightable ? <SyntaxHighlighter code={code} /> : <code className="text-muted-foreground">{code}</code>}
      </pre>
      <button
        onClick={handleCopy}
        className="absolute top-3 right-3 p-2 rounded-md bg-white/5 hover:bg-white/10 border border-white/10 transition-all opacity-0 group-hover:opacity-100"
        aria-label="Copy code"
      >
        {copied ? <Check className="w-4 h-4 text-green-500" /> : <Copy className="w-4 h-4 text-muted-foreground" />}
      </button>
    </div>
  )
}
