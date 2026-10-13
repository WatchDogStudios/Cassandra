// Markdown content for each documentation section
export const docsMarkdown = {
  introduction: `# Introduction

Infiner provides a unified API to access 50+ AI models with industry-leading latency and 99.99% uptime. Our intelligent routing automatically selects the best model for your use case.

## Key Features

- **12ms Latency** - Average response time across all models
- **50+ Models** - Access GPT-4, Claude, Llama, and more
- **One API Key** - Single key for all model providers`,

  quickstart: `# Quickstart

Get started with Infiner in under 5 minutes. Install our SDK and make your first API call.

## 1. Install the SDK

\`\`\`bash
npm install @infiner/sdk
\`\`\`

## 2. Initialize the client

\`\`\`typescript
import { Infiner } from '@infiner/sdk'

const infiner = new Infiner({
  apiKey: 'YOUR_API_KEY'
})
\`\`\`

## 3. Make your first request

\`\`\`typescript
const response = await infiner.chat.completions.create({
  model: 'gpt-4-turbo',
  messages: [
    { role: 'user', content: 'Hello, how are you?' }
  ]
})

console.log(response.choices[0].message.content)
\`\`\``,

  authentication: `# Authentication

All API requests require an API key. You can create and manage API keys in your dashboard.

## Using your API key

Include your API key in the \`Authorization\` header:

\`\`\`bash
curl https://api.infiner.ai/v1/chat/completions \\
  -H "Authorization: Bearer YOUR_API_KEY" \\
  -H "Content-Type: application/json" \\
  -d '{
    "model": "gpt-4-turbo",
    "messages": [{"role": "user", "content": "Hello!"}]
  }'
\`\`\``,

  "chat-completions": `# Chat Completions

Create chat completions using any supported model. The API is compatible with the OpenAI format.

## Request

\`\`\`typescript
const response = await infiner.chat.completions.create({
  model: 'claude-3-opus',
  messages: [
    { role: 'system', content: 'You are a helpful assistant.' },
    { role: 'user', content: 'Explain quantum computing.' }
  ],
  temperature: 0.7,
  max_tokens: 1000
})
\`\`\`

## Response

\`\`\`json
{
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
}
\`\`\``,

  embeddings: `# Embeddings

Generate vector embeddings for text using models like text-embedding-3-large.

\`\`\`typescript
const embedding = await infiner.embeddings.create({
  model: 'text-embedding-3-large',
  input: 'The quick brown fox jumps over the lazy dog'
})

console.log(embedding.data[0].embedding) // [0.0023, -0.0045, ...]
\`\`\``,

  models: `# Models

List available models and retrieve model information.

\`\`\`typescript
// List all available models
const models = await infiner.models.list()

// Get specific model info
const model = await infiner.models.retrieve('gpt-4-turbo')
console.log(model.context_length) // 128000
\`\`\``,

  "model-routing": `# Model Routing

Let Infiner automatically select the best model for your request based on cost, latency, and capability.

\`\`\`typescript
const response = await infiner.chat.completions.create({
  model: 'auto', // Infiner selects the best model
  messages: [
    { role: 'user', content: 'Write a haiku about coding' }
  ],
  routing: {
    optimize: 'cost', // 'cost' | 'latency' | 'quality'
    fallback: ['gpt-4-turbo', 'claude-3-opus']
  }
})
\`\`\``,

  streaming: `# Streaming

Stream responses token by token for real-time user experiences.

\`\`\`typescript
const stream = await infiner.chat.completions.create({
  model: 'gpt-4-turbo',
  messages: [{ role: 'user', content: 'Tell me a story' }],
  stream: true
})

for await (const chunk of stream) {
  process.stdout.write(chunk.choices[0]?.delta?.content || '')
}
\`\`\``,

  "function-calling": `# Function Calling

Enable models to call functions and interact with external tools.

\`\`\`typescript
const response = await infiner.chat.completions.create({
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
})
\`\`\``,

  "api-keys": `# API Keys

Manage your API keys securely. Never expose keys in client-side code.

> **Security Warning:** Keep your API keys secure. Do not share them or expose them in browser code.

\`\`\`bash
# Store your API key in environment variables
export INFINER_API_KEY="inf_sk_..."
\`\`\``,

  "rate-limits": `# Rate Limits

Rate limits vary by plan. Check response headers for your current usage.

| Plan | Requests/min | Tokens/min |
|------|--------------|------------|
| Developer | 60 | 40,000 |
| Pro | 500 | 200,000 |
| Enterprise | Unlimited | Custom |

## Rate limit headers

\`\`\`http
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1703654400
\`\`\``,
}

export const fullDocsMarkdown = Object.values(docsMarkdown).join("\n\n---\n\n")

export type DocsSectionId = keyof typeof docsMarkdown
