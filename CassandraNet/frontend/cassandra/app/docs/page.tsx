"use client"

import { useState, useEffect } from "react"
import Link from "next/link"
import { ChevronRight, Zap, Database, Key } from "lucide-react"
import { Button } from "@/components/ui/button"
import { DocsHeader } from "@/components/docs/docs-header"
import { DocsSidebar } from "@/components/docs/docs-sidebar"
import { CodeBlock } from "@/components/docs/code-block"
import { CopyPageButton } from "@/components/docs/copy-page-button"
import type { DocsSectionId } from "@/lib/docs-content"

export default function DocsPage() {
  const [activeSection, setActiveSection] = useState<DocsSectionId>("introduction")

  useEffect(() => {
    window.scrollTo(0, 0)
  }, [])

  return (
    <div className="min-h-screen bg-background">
      <DocsHeader />

      <div className="flex pt-16">
        <DocsSidebar activeSection={activeSection} onSectionChange={setActiveSection} />

        <main className="flex-1 lg:ml-64 px-6 lg:px-12 py-12 max-w-4xl">
          {/* Introduction */}
          <section id="introduction" className="mb-16">
            <div className="flex items-center gap-2 text-sm text-muted-foreground mb-4">
              <Link href="/" className="hover:text-foreground">
                Home
              </Link>
              <ChevronRight className="w-4 h-4" />
              <span className="text-foreground">Documentation</span>
            </div>
            <div className="flex items-start justify-between gap-4 mb-4">
              <h1 className="text-4xl font-bold">Introduction</h1>
              <CopyPageButton />
            </div>
            <p className="text-lg text-muted-foreground mb-8">
              Infiner provides a unified API to access 50+ AI models with industry-leading latency and 99.99% uptime.
              Our intelligent routing automatically selects the best model for your use case.
            </p>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
              <div className="bg-surface-elevated border border-white/10 rounded-lg p-4">
                <Zap className="w-8 h-8 text-primary mb-3" />
                <h3 className="font-semibold mb-1">12ms Latency</h3>
                <p className="text-sm text-muted-foreground">Average response time across all models</p>
              </div>
              <div className="bg-surface-elevated border border-white/10 rounded-lg p-4">
                <Database className="w-8 h-8 text-primary mb-3" />
                <h3 className="font-semibold mb-1">50+ Models</h3>
                <p className="text-sm text-muted-foreground">Access GPT-4, Claude, Llama, and more</p>
              </div>
              <div className="bg-surface-elevated border border-white/10 rounded-lg p-4">
                <Key className="w-8 h-8 text-primary mb-3" />
                <h3 className="font-semibold mb-1">One API Key</h3>
                <p className="text-sm text-muted-foreground">Single key for all model providers</p>
              </div>
            </div>
          </section>

          {/* Quickstart */}
          <section id="quickstart" className="mb-16">
            <h2 className="text-2xl font-bold mb-4">Quickstart</h2>
            <p className="text-muted-foreground mb-6">
              Get started with Infiner in under 5 minutes. Install our SDK and make your first API call.
            </p>
            <h3 className="text-lg font-semibold mb-3">1. Install the SDK</h3>
            <CodeBlock code="npm install @infiner/sdk" language="bash" />
            <h3 className="text-lg font-semibold mt-6 mb-3">2. Initialize the client</h3>
            <CodeBlock
              language="typescript"
              code={`import { Infiner } from '@infiner/sdk'

const infiner = new Infiner({
  apiKey: 'YOUR_API_KEY'
})`}
            />
            <h3 className="text-lg font-semibold mt-6 mb-3">3. Make your first request</h3>
            <CodeBlock
              language="typescript"
              code={`const response = await infiner.chat.completions.create({
  model: 'gpt-4-turbo',
  messages: [
    { role: 'user', content: 'Hello, how are you?' }
  ]
})

console.log(response.choices[0].message.content)`}
            />
          </section>

          {/* Authentication */}
          <section id="authentication" className="mb-16">
            <h2 className="text-2xl font-bold mb-4">Authentication</h2>
            <p className="text-muted-foreground mb-6">
              All API requests require an API key. You can create and manage API keys in your dashboard.
            </p>
            <h3 className="text-lg font-semibold mb-3">Using your API key</h3>
            <p className="text-muted-foreground mb-4">
              Include your API key in the{" "}
              <code className="bg-surface-dark px-2 py-0.5 rounded text-sm">Authorization</code> header:
            </p>
            <CodeBlock
              language="bash"
              code={`curl https://api.infiner.ai/v1/chat/completions \\
  -H "Authorization: Bearer YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "model": "gpt-4-turbo",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'`}
            />
          </section>

          {/* Chat Completions */}
          <section id="chat-completions" className="mb-16">
            <h2 className="text-2xl font-bold mb-4">Chat Completions</h2>
            <p className="text-muted-foreground mb-6">
              Create chat completions using any supported model. The API is compatible with the OpenAI format.
            </p>
            <h3 className="text-lg font-semibold mb-3">Request</h3>
            <CodeBlock
              language="typescript"
              code={`const response = await infiner.chat.completions.create({
  model: 'claude-3-opus',
  messages: [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'Explain quantum computing.' }
  ],
  temperature: 0.7,
  max_tokens: 1000
})`}
            />
            <h3 className="text-lg font-semibold mt-6 mb-3">Response</h3>
            <CodeBlock
              language="json"
              code={`{
  "id": "chatcmpl-abc123",
  "object": "chat.completion",
  "created": 1703654321,
  "model": "claude-3-opus",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Quantum computing is..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 25,
    "completion_tokens": 150,
    "total_tokens": 175
  }
}`}
            />
          </section>

          {/* Embeddings */}
          <section id="embeddings" className="mb-16">
            <h2 className="text-2xl font-bold mb-4">Embeddings</h2>
            <p className="text-muted-foreground mb-6">
              Generate vector embeddings for text using models like text-embedding-3-large.
            </p>
            <CodeBlock
              language="typescript"
              code={`const embedding = await infiner.embeddings.create({
  model: 'text-embedding-3-large',
  input: 'The quick brown fox jumps over the lazy dog'
})

console.log(embedding.data[0].embedding) // [0.0023, -0.0045, ...]`}
            />
          </section>

          {/* Models */}
          <section id="models" className="mb-16">
            <h2 className="text-2xl font-bold mb-4">Models</h2>
            <p className="text-muted-foreground mb-6">List available models and retrieve model information.</p>
            <CodeBlock
              language="typescript"
              code={`// List all available models
const models = await infiner.models.list()

// Get specific model info
const model = await infiner.models.retrieve('gpt-4-turbo')
console.log(model.context_length) // 128000`}
            />
            <div className="mt-6">
              <Link href="/models" className="text-primary hover:underline inline-flex items-center gap-1">
                View all available models <ChevronRight className="w-4 h-4" />
              </Link>
            </div>
          </section>

          {/* Model Routing */}
          <section id="model-routing" className="mb-16">
            <h2 className="text-2xl font-bold mb-4">Model Routing</h2>
            <p className="text-muted-foreground mb-6">
              Let Infiner automatically select the best model for your request based on cost, latency, and capability.
            </p>
            <CodeBlock
              language="typescript"
              code={`const response = await infiner.chat.completions.create({
  model: 'auto', // Infiner selects the best model
  messages: [
    { role: 'user', content: 'Write a haiku about coding' }
  ],
  routing: {
    optimize: 'cost', // 'cost' | 'latency' | 'quality'
    fallback: ['gpt-4-turbo', 'claude-3-opus']
  }
})`}
            />
          </section>

          {/* Streaming */}
          <section id="streaming" className="mb-16">
            <h2 className="text-2xl font-bold mb-4">Streaming</h2>
            <p className="text-muted-foreground mb-6">
              Stream responses token by token for real-time user experiences.
            </p>
            <CodeBlock
              language="typescript"
              code={`const stream = await infiner.chat.completions.create({
  model: 'gpt-4-turbo',
  messages: [{ role: 'user', content: 'Tell me a story' }],
  stream: true
})

for await (const chunk of stream) {
  process.stdout.write(chunk.choices[0]?.delta?.content || '')
}`}
            />
          </section>

          {/* Function Calling */}
          <section id="function-calling" className="mb-16">
            <h2 className="text-2xl font-bold mb-4">Function Calling</h2>
            <p className="text-muted-foreground mb-6">
              Enable models to call functions and interact with external tools.
            </p>
            <CodeBlock
              language="typescript"
              code={`const response = await infiner.chat.completions.create({
  model: 'gpt-4-turbo',
  messages: [{ role: 'user', content: 'What is the weather in Tokyo?' }],
  tools: [
    {
      type: 'function',
      function: {
        name: 'get_weather',
        description: 'Get current weather for a location',
        parameters: {
          type: 'object',
          properties: {
            location: { type: 'string', description: 'City name' }
          },
          required: ['location']
        }
      }
    }
  ]
})`}
            />
          </section>

          {/* API Keys */}
          <section id="api-keys" className="mb-16">
            <h2 className="text-2xl font-bold mb-4">API Keys</h2>
            <p className="text-muted-foreground mb-6">
              Manage your API keys securely. Never expose keys in client-side code.
            </p>
            <div className="bg-surface-elevated border border-yellow-500/20 rounded-lg p-4 mb-6">
              <p className="text-sm text-yellow-500">
                <strong>Security Warning:</strong> Keep your API keys secure. Do not share them or expose them in
                browser code.
              </p>
            </div>
            <CodeBlock
              language="bash"
              code={`# Store your API key in environment variables
export INFINER_API_KEY="inf_sk_..."`}
            />
          </section>

          {/* Rate Limits */}
          <section id="rate-limits" className="mb-16">
            <h2 className="text-2xl font-bold mb-4">Rate Limits</h2>
            <p className="text-muted-foreground mb-6">
              Rate limits vary by plan. Check response headers for your current usage.
            </p>
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b border-white/10">
                    <th className="text-left py-3 px-4 font-semibold">Plan</th>
                    <th className="text-left py-3 px-4 font-semibold">Requests/min</th>
                    <th className="text-left py-3 px-4 font-semibold">Tokens/min</th>
                  </tr>
                </thead>
                <tbody className="text-muted-foreground">
                  <tr className="border-b border-white/10">
                    <td className="py-3 px-4">Developer</td>
                    <td className="py-3 px-4">60</td>
                    <td className="py-3 px-4">40,000</td>
                  </tr>
                  <tr className="border-b border-white/10">
                    <td className="py-3 px-4">Pro</td>
                    <td className="py-3 px-4">500</td>
                    <td className="py-3 px-4">200,000</td>
                  </tr>
                  <tr>
                    <td className="py-3 px-4">Enterprise</td>
                    <td className="py-3 px-4">Unlimited</td>
                    <td className="py-3 px-4">Custom</td>
                  </tr>
                </tbody>
              </table>
            </div>
            <h3 className="text-lg font-semibold mt-6 mb-3">Rate limit headers</h3>
            <CodeBlock
              language="http"
              code={`X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1703654400`}
            />
          </section>

          {/* Footer navigation */}
          <div className="border-t border-white/10 pt-8 mt-16">
            <div className="flex justify-between items-center">
              <div>
                <p className="text-sm text-muted-foreground mb-1">Questions?</p>
                <Link href="/contact" className="text-primary hover:underline">
                  Contact our team
                </Link>
              </div>
              <Link href="/signup">
                <Button className="bg-primary hover:bg-primary/90">Get your API key</Button>
              </Link>
            </div>
          </div>
        </main>
      </div>
    </div>
  )
}
