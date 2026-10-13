"use client"

// Brand-specific syntax highlighting using Infiner theme colors
// Parses JavaScript/TypeScript code and applies tokenized colors

interface Token {
  type:
    | "keyword"
    | "string"
    | "number"
    | "comment"
    | "function"
    | "property"
    | "operator"
    | "punctuation"
    | "variable"
    | "constant"
    | "text"
  value: string
}

const KEYWORDS = [
  "const",
  "let",
  "var",
  "function",
  "return",
  "if",
  "else",
  "for",
  "while",
  "do",
  "switch",
  "case",
  "break",
  "continue",
  "try",
  "catch",
  "finally",
  "throw",
  "async",
  "await",
  "import",
  "export",
  "from",
  "default",
  "class",
  "extends",
  "new",
  "this",
  "super",
  "static",
  "get",
  "set",
  "typeof",
  "instanceof",
  "in",
  "of",
  "true",
  "false",
  "null",
  "undefined",
  "void",
  "delete",
  "yield",
]

const CONSTANTS = ["true", "false", "null", "undefined", "NaN", "Infinity"]

export function tokenize(code: string): Token[] {
  const tokens: Token[] = []
  let i = 0

  while (i < code.length) {
    // Skip whitespace but preserve it
    if (/\s/.test(code[i])) {
      let ws = ""
      while (i < code.length && /\s/.test(code[i])) {
        ws += code[i]
        i++
      }
      tokens.push({ type: "text", value: ws })
      continue
    }

    // Single-line comments
    if (code[i] === "/" && code[i + 1] === "/") {
      let comment = ""
      while (i < code.length && code[i] !== "\n") {
        comment += code[i]
        i++
      }
      tokens.push({ type: "comment", value: comment })
      continue
    }

    // Multi-line comments
    if (code[i] === "/" && code[i + 1] === "*") {
      let comment = ""
      while (i < code.length && !(code[i] === "*" && code[i + 1] === "/")) {
        comment += code[i]
        i++
      }
      comment += "*/"
      i += 2
      tokens.push({ type: "comment", value: comment })
      continue
    }

    // Strings (single, double, template)
    if (code[i] === '"' || code[i] === "'" || code[i] === "`") {
      const quote = code[i]
      let str = quote
      i++
      while (i < code.length && code[i] !== quote) {
        if (code[i] === "\\" && i + 1 < code.length) {
          str += code[i] + code[i + 1]
          i += 2
        } else {
          str += code[i]
          i++
        }
      }
      str += code[i] || ""
      i++
      tokens.push({ type: "string", value: str })
      continue
    }

    // Numbers
    if (/\d/.test(code[i]) || (code[i] === "." && /\d/.test(code[i + 1]))) {
      let num = ""
      while (i < code.length && /[\d.exXabcdefABCDEF]/.test(code[i])) {
        num += code[i]
        i++
      }
      tokens.push({ type: "number", value: num })
      continue
    }

    // Identifiers and keywords
    if (/[a-zA-Z_$]/.test(code[i])) {
      let ident = ""
      while (i < code.length && /[a-zA-Z0-9_$]/.test(code[i])) {
        ident += code[i]
        i++
      }

      // Check if it's followed by a parenthesis (function call)
      let isFunction = false
      let j = i
      while (j < code.length && /\s/.test(code[j])) j++
      if (code[j] === "(") isFunction = true

      if (CONSTANTS.includes(ident)) {
        tokens.push({ type: "constant", value: ident })
      } else if (KEYWORDS.includes(ident)) {
        tokens.push({ type: "keyword", value: ident })
      } else if (isFunction) {
        tokens.push({ type: "function", value: ident })
      } else if (ident === ident.toUpperCase() && ident.length > 1) {
        tokens.push({ type: "constant", value: ident })
      } else {
        tokens.push({ type: "variable", value: ident })
      }
      continue
    }

    // Operators
    if (/[+\-*/%=<>!&|^~?:]/.test(code[i])) {
      let op = code[i]
      i++
      // Handle multi-char operators
      while (i < code.length && /[+\-*/%=<>!&|^~?:]/.test(code[i])) {
        op += code[i]
        i++
      }
      tokens.push({ type: "operator", value: op })
      continue
    }

    // Punctuation
    if (/[{}[\]();,.]/.test(code[i])) {
      tokens.push({ type: "punctuation", value: code[i] })
      i++
      continue
    }

    // Default: any other character
    tokens.push({ type: "text", value: code[i] })
    i++
  }

  return tokens
}

// Color mapping using CSS variables for brand consistency
const TOKEN_COLORS: Record<Token["type"], string> = {
  keyword: "text-primary", // Blue - #225EDF
  string: "text-emerald-400", // Green for strings
  number: "text-amber-400", // Amber for numbers
  comment: "text-muted-foreground italic", // Muted gray
  function: "text-sky-400", // Light blue for functions
  property: "text-violet-400", // Violet for properties
  operator: "text-rose-400", // Rose for operators
  punctuation: "text-muted-foreground", // Muted for punctuation
  variable: "text-foreground", // White for variables
  constant: "text-orange-400", // Orange for constants
  text: "text-foreground", // Default text
}

interface SyntaxHighlighterProps {
  code: string
  className?: string
}

export function SyntaxHighlighter({ code, className = "" }: SyntaxHighlighterProps) {
  const tokens = tokenize(code)

  return (
    <pre className={`font-mono text-sm overflow-x-auto ${className}`}>
      <code>
        {tokens.map((token, i) => (
          <span key={i} className={TOKEN_COLORS[token.type]}>
            {token.value}
          </span>
        ))}
      </code>
    </pre>
  )
}
