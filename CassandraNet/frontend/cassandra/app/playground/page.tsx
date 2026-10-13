"use client"

import type React from "react"

import { useState, useRef, useEffect } from "react"
import { Navbar } from "@/components/navbar"
import { Footer } from "@/components/footer"
import { Button } from "@/components/ui/button"
import { Card } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Slider } from "@/components/ui/slider"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { SyntaxHighlighter } from "@/lib/syntax-highlighter"
import {
  Play,
  Copy,
  Check,
  Settings2,
  Sparkles,
  Clock,
  Zap,
  RotateCcw,
  ChevronDown,
  Code,
  MessageSquare,
} from "lucide-react"

interface Message {
  role: "user" | "assistant" | "system"
  content: string
}

interface Model {
  id: string
  name: string
  provider: string
  maxTokens: number
}

const models: Model[] = [
  { id: "gpt-4-turbo", name: "GPT-4 Turbo", provider: "OpenAI", maxTokens: 128000 },
  { id: "gpt-4o", name: "GPT-4o", provider: "OpenAI", maxTokens: 128000 },
  { id: "claude-3-5-sonnet", name: "Claude 3.5 Sonnet", provider: "Anthropic", maxTokens: 200000 },
  { id: "claude-3-opus", name: "Claude 3 Opus", provider: "Anthropic", maxTokens: 200000 },
  { id: "gemini-1-5-pro", name: "Gemini 1.5 Pro", provider: "Google", maxTokens: 1000000 },
  { id: "llama-3-1-405b", name: "Llama 3.1 405B", provider: "Meta", maxTokens: 128000 },
  { id: "mistral-large-2", name: "Mistral Large 2", provider: "Mistral AI", maxTokens: 128000 },
]

const presetPrompts = [
  { label: "Explain code", prompt: "Explain what this code does and suggest improvements:" },
  { label: "Write a function", prompt: "Write a TypeScript function that:" },
  { label: "Debug this", prompt: "Help me debug this error:" },
  { label: "Summarize", prompt: "Summarize the following text:" },
]

// Mock streaming response
const mockResponses: Record<string, string> = {
  default: `I'd be happy to help you with that! Here's a detailed response based on your query.

When working with modern web development, it's important to consider several factors:

1. **Performance**: Always optimize for the best user experience
2. **Accessibility**: Ensure your application is usable by everyone  
3. **Maintainability**: Write clean, well-documented code

Here's an example implementation:

\`\`\`typescript
async function fetchData<T>(url: string): Promise<T> {
  const response = await fetch(url, {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': \`Bearer YOUR_API_KEY\`
    }
  });
  
  if (!response.ok) {
    throw new Error(\`HTTP error! status: \${response.status}\`);
  }
  
  return response.json();
}
\`\`\`

This approach ensures type safety while maintaining flexibility. Let me know if you need any clarification!`,
}

export default function PlaygroundPage() {
  const [selectedModel, setSelectedModel] = useState(models[0])
  const [messages, setMessages] = useState<Message[]>([])
  const [inputValue, setInputValue] = useState("")
  const [systemPrompt, setSystemPrompt] = useState("You are a helpful AI assistant.")
  const [isStreaming, setIsStreaming] = useState(false)
  const [streamedContent, setStreamedContent] = useState("")
  const [showSettings, setShowSettings] = useState(false)
  const [copied, setCopied] = useState(false)
  const [temperature, setTemperature] = useState([0.7])
  const [maxTokens, setMaxTokens] = useState([2048])
  const [latency, setLatency] = useState<number | null>(null)
  const [tokensGenerated, setTokensGenerated] = useState(0)
  const [viewMode, setViewMode] = useState<"chat" | "code">("chat")
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  // Scroll to bottom on new messages
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" })
  }, [messages, streamedContent])

  // Scroll to top on mount
  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  // Auto-resize textarea
  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto"
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 200)}px`
    }
  }, [inputValue])

  const simulateStreaming = async (response: string) => {
    const startTime = Date.now()
    setIsStreaming(true)
    setStreamedContent("")
    setTokensGenerated(0)

    const words = response.split(" ")
    let currentContent = ""
    let tokenCount = 0

    for (let i = 0; i < words.length; i++) {
      await new Promise((resolve) => setTimeout(resolve, 20 + Math.random() * 30))
      currentContent += (i === 0 ? "" : " ") + words[i]
      tokenCount += 1
      setStreamedContent(currentContent)
      setTokensGenerated(tokenCount)
    }

    const endTime = Date.now()
    setLatency(endTime - startTime)
    setIsStreaming(false)
    setMessages((prev) => [...prev, { role: "assistant", content: response }])
    setStreamedContent("")
  }

  const handleSubmit = async () => {
    if (!inputValue.trim() || isStreaming) return

    const userMessage: Message = { role: "user", content: inputValue }
    setMessages((prev) => [...prev, userMessage])
    setInputValue("")

    // Get mock response
    const response = mockResponses.default
    await simulateStreaming(response)
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault()
      handleSubmit()
    }
  }

  const handleCopy = async () => {
    const content = messages.map((m) => `${m.role}: ${m.content}`).join("\n\n")
    await navigator.clipboard.writeText(content)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const handleReset = () => {
    setMessages([])
    setStreamedContent("")
    setLatency(null)
    setTokensGenerated(0)
  }

  const generateCodeSnippet = () => {
    return `import Infiner from '@infiner/sdk';

const client = new Infiner({
  apiKey: 'YOUR_API_KEY'
});

const response = await client.chat.completions.create({
  model: '${selectedModel.id}',
  messages: [
    { role: 'system', content: '${systemPrompt}' },
    ${messages.map((m) => `{ role: '${m.role}', content: '${m.content.replace(/'/g, "\\'")}' }`).join(",\n    ")}
  ],
  temperature: ${temperature[0]},
  max_tokens: ${maxTokens[0]}
});

console.log(response.choices[0].message.content);`
  }

  return (
    <div className="min-h-screen bg-background flex flex-col">
      <Navbar />

      <main className="flex-1 pt-20 pb-4 flex flex-col min-h-[calc(100vh-5rem)]">
        <div className="container mx-auto px-4 flex flex-col flex-1">
          {/* Header */}
          <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4 mb-6">
            <div>
              <h1 className="text-2xl md:text-3xl font-bold text-foreground">API Playground</h1>
              <p className="text-muted-foreground mt-1">Test models in real-time. No API key required.</p>
            </div>

            <div className="flex items-center gap-2">
              {/* View Mode Toggle */}
              <div className="flex items-center bg-card border border-white/10 rounded-lg p-1">
                <button
                  onClick={() => setViewMode("chat")}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm transition-colors ${
                    viewMode === "chat" ? "bg-primary text-white" : "text-muted-foreground hover:text-foreground"
                  }`}
                >
                  <MessageSquare className="h-4 w-4" />
                  Chat
                </button>
                <button
                  onClick={() => setViewMode("code")}
                  className={`flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm transition-colors ${
                    viewMode === "code" ? "bg-primary text-white" : "text-muted-foreground hover:text-foreground"
                  }`}
                >
                  <Code className="h-4 w-4" />
                  Code
                </button>
              </div>
            </div>
          </div>

          <div className="grid lg:grid-cols-[1fr_320px] gap-6 flex-1">
            {/* Main Panel */}
            <div className="flex flex-col min-h-[500px]">
              {/* Model Selector Bar */}
              <Card className="bg-card border-white/10 p-4 mb-4">
                <div className="flex flex-wrap items-center gap-4">
                  <div className="flex-1 min-w-[200px]">
                    <Select
                      value={selectedModel.id}
                      onValueChange={(value) => {
                        const model = models.find((m) => m.id === value)
                        if (model) setSelectedModel(model)
                      }}
                    >
                      <SelectTrigger className="bg-background border-white/10">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent className="bg-card border-white/10">
                        {models.map((model) => (
                          <SelectItem key={model.id} value={model.id}>
                            <div className="flex items-center gap-2">
                              <span>{model.name}</span>
                              <Badge variant="secondary" className="text-xs bg-primary/10 text-primary">
                                {model.provider}
                              </Badge>
                            </div>
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>

                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowSettings(!showSettings)}
                    className="border-white/10 bg-card hover:bg-card/80"
                  >
                    <Settings2 className="h-4 w-4 mr-2" />
                    Settings
                    <ChevronDown className={`h-3 w-3 ml-1 transition-transform ${showSettings ? "rotate-180" : ""}`} />
                  </Button>

                  {messages.length > 0 && (
                    <>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={handleCopy}
                        className="border-white/10 bg-card hover:bg-card/80"
                      >
                        {copied ? <Check className="h-4 w-4 mr-2" /> : <Copy className="h-4 w-4 mr-2" />}
                        {copied ? "Copied" : "Copy"}
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={handleReset}
                        className="border-white/10 bg-card hover:bg-card/80"
                      >
                        <RotateCcw className="h-4 w-4 mr-2" />
                        Reset
                      </Button>
                    </>
                  )}
                </div>

                {/* Expandable Settings */}
                {showSettings && (
                  <div className="mt-4 pt-4 border-t border-white/10 grid md:grid-cols-2 gap-6">
                    <div>
                      <label className="text-sm text-muted-foreground mb-2 block">System Prompt</label>
                      <textarea
                        value={systemPrompt}
                        onChange={(e) => setSystemPrompt(e.target.value)}
                        className="w-full bg-background border border-white/10 rounded-lg p-3 text-sm text-foreground resize-none h-20"
                        placeholder="You are a helpful assistant..."
                      />
                    </div>
                    <div className="space-y-4">
                      <div>
                        <div className="flex items-center justify-between mb-2">
                          <label className="text-sm text-muted-foreground">Temperature</label>
                          <span className="text-sm text-foreground">{temperature[0]}</span>
                        </div>
                        <Slider
                          value={temperature}
                          onValueChange={setTemperature}
                          max={2}
                          min={0}
                          step={0.1}
                          className="w-full"
                        />
                      </div>
                      <div>
                        <div className="flex items-center justify-between mb-2">
                          <label className="text-sm text-muted-foreground">Max Tokens</label>
                          <span className="text-sm text-foreground">{maxTokens[0]}</span>
                        </div>
                        <Slider
                          value={maxTokens}
                          onValueChange={setMaxTokens}
                          max={4096}
                          min={256}
                          step={256}
                          className="w-full"
                        />
                      </div>
                    </div>
                  </div>
                )}
              </Card>

              {/* Chat / Code View */}
              {viewMode === "chat" ? (
                <Card className="bg-card border-white/10 flex-1 flex flex-col min-h-0">
                  {/* Messages Area */}
                  <div
                    className={`flex-1 p-4 space-y-4 min-h-0 ${messages.length === 0 && !streamedContent ? "flex items-center justify-center" : "overflow-y-auto"}`}
                  >
                    {messages.length === 0 && !streamedContent && (
                      <div className="flex flex-col items-center justify-center text-center">
                        <div className="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center mb-4">
                          <Sparkles className="h-8 w-8 text-primary" />
                        </div>
                        <h3 className="text-lg font-medium text-foreground mb-2">Start a conversation</h3>
                        <p className="text-muted-foreground text-sm max-w-md mb-6">
                          Test any model with your own prompts. Results stream in real-time.
                        </p>
                        <div className="flex flex-wrap gap-2 justify-center">
                          {presetPrompts.map((preset) => (
                            <Button
                              key={preset.label}
                              variant="outline"
                              size="sm"
                              onClick={() => setInputValue(preset.prompt)}
                              className="border-white/10 bg-card hover:bg-card/80"
                            >
                              {preset.label}
                            </Button>
                          ))}
                        </div>
                      </div>
                    )}

                    {messages.map((message, index) => (
                      <div key={index} className={`flex ${message.role === "user" ? "justify-end" : "justify-start"}`}>
                        <div
                          className={`max-w-[80%] rounded-xl px-4 py-3 ${
                            message.role === "user"
                              ? "bg-primary text-white"
                              : "bg-surface-elevated border border-white/5"
                          }`}
                        >
                          <pre className="whitespace-pre-wrap font-sans text-sm leading-relaxed">{message.content}</pre>
                        </div>
                      </div>
                    ))}

                    {/* Streaming Response */}
                    {streamedContent && (
                      <div className="flex justify-start">
                        <div className="max-w-[80%] rounded-xl px-4 py-3 bg-surface-elevated border border-white/5">
                          <pre className="whitespace-pre-wrap font-sans text-sm leading-relaxed">
                            {streamedContent}
                            <span className="inline-block w-2 h-4 bg-primary ml-1 animate-pulse" />
                          </pre>
                        </div>
                      </div>
                    )}

                    <div ref={messagesEndRef} />
                  </div>

                  {/* Input Area */}
                  <div className="p-4 border-t border-white/10">
                    <div className="flex gap-2">
                      <textarea
                        ref={textareaRef}
                        value={inputValue}
                        onChange={(e) => setInputValue(e.target.value)}
                        onKeyDown={handleKeyDown}
                        placeholder="Type your message... (Shift+Enter for new line)"
                        className="flex-1 bg-background border border-white/10 rounded-lg px-4 py-3 text-sm text-foreground resize-none min-h-[48px] max-h-[200px] focus:outline-none focus:ring-0 focus:border-primary"
                        rows={1}
                        aria-label="Message input"
                      />
                      <Button
                        onClick={handleSubmit}
                        disabled={!inputValue.trim() || isStreaming}
                        className="bg-primary hover:bg-primary/90 text-white px-4"
                        aria-label={isStreaming ? "Sending message" : "Send message"}
                      >
                        {isStreaming ? (
                          <div className="h-5 w-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                        ) : (
                          <Play className="h-5 w-5" />
                        )}
                      </Button>
                    </div>
                  </div>
                </Card>
              ) : (
                <Card className="bg-card border-white/10 flex-1 flex flex-col min-h-0">
                  <div className="flex-1 overflow-auto p-4 min-h-0">
                    <div className="flex items-center justify-between mb-4">
                      <span className="text-sm text-muted-foreground">Generated API Code</span>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={async () => {
                          await navigator.clipboard.writeText(generateCodeSnippet())
                          setCopied(true)
                          setTimeout(() => setCopied(false), 2000)
                        }}
                        className="border-white/10 bg-card hover:bg-card/80"
                      >
                        {copied ? <Check className="h-4 w-4 mr-2" /> : <Copy className="h-4 w-4 mr-2" />}
                        {copied ? "Copied" : "Copy"}
                      </Button>
                    </div>
                    <SyntaxHighlighter code={generateCodeSnippet()} />
                  </div>
                </Card>
              )}
            </div>

            {/* Sidebar - Stats & Info */}
            <div className="space-y-4">
              {/* Live Stats */}
              <Card className="bg-card border-white/10 p-4">
                <h3 className="text-sm font-medium text-foreground mb-4">Live Stats</h3>
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 text-muted-foreground">
                      <Clock className="h-4 w-4" />
                      <span className="text-sm">Latency</span>
                    </div>
                    <span className="text-sm font-medium text-foreground">{latency ? `${latency}ms` : "—"}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 text-muted-foreground">
                      <Zap className="h-4 w-4" />
                      <span className="text-sm">Tokens Generated</span>
                    </div>
                    <span className="text-sm font-medium text-foreground">{tokensGenerated || "—"}</span>
                  </div>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2 text-muted-foreground">
                      <Sparkles className="h-4 w-4" />
                      <span className="text-sm">Model</span>
                    </div>
                    <Badge variant="secondary" className="text-xs bg-primary/10 text-primary">
                      {selectedModel.name}
                    </Badge>
                  </div>
                </div>
              </Card>

              {/* Model Info */}
              <Card className="bg-card border-white/10 p-4">
                <h3 className="text-sm font-medium text-foreground mb-4">Model Info</h3>
                <div className="space-y-3 text-sm">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Provider</span>
                    <span className="text-foreground">{selectedModel.provider}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Context Window</span>
                    <span className="text-foreground">{(selectedModel.maxTokens / 1000).toFixed(0)}K tokens</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">Max Output</span>
                    <span className="text-foreground">{maxTokens[0]} tokens</span>
                  </div>
                </div>
              </Card>

              {/* Quick Actions */}
              <Card className="bg-card border-white/10 p-4">
                <h3 className="text-sm font-medium text-foreground mb-4">Quick Actions</h3>
                <div className="space-y-2">
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full border-white/10 bg-card hover:bg-card/80 justify-start"
                    asChild
                  >
                    <a href="https://ai-sdk.dev/docs" target="_blank" rel="noopener noreferrer">
                      <Code className="h-4 w-4 mr-2" />
                      View API Docs
                    </a>
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full border-white/10 bg-card hover:bg-card/80 justify-start"
                    asChild
                  >
                    <a
                      href="https://ai-sdk.dev/docs/foundations/providers-and-models"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      <Sparkles className="h-4 w-4 mr-2" />
                      Compare Models
                    </a>
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full border-white/10 bg-card hover:bg-card/80 justify-start"
                    asChild
                  >
                    <a href="https://ai-sdk.dev/docs/getting-started" target="_blank" rel="noopener noreferrer">
                      <Zap className="h-4 w-4 mr-2" />
                      Get Started
                    </a>
                  </Button>
                </div>
              </Card>
            </div>
          </div>
        </div>
      </main>

      <Footer />
    </div>
  )
}
