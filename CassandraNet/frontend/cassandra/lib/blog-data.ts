export interface BlogPost {
  slug: string
  title: string
  excerpt: string
  content: string
  author: {
    name: string
    role: string
    avatar: string
  }
  category: string
  publishedAt: string
  readTime: string
  coverImage: string
  featured?: boolean
}

export const blogPosts: BlogPost[] = [
  {
    slug: "scaling-llm-inference-to-millions",
    title: "Scaling LLM Inference to Millions of Requests",
    excerpt:
      "Learn how we architected our infrastructure to handle millions of inference requests per second while maintaining sub-100ms latency.",
    content: `
# Scaling LLM Inference to Millions of Requests

When we set out to build Infiner, we knew that traditional scaling approaches wouldn't cut it. Large Language Models present unique challenges that require rethinking infrastructure from the ground up.

## The Challenge

Modern LLMs are compute-intensive beasts. A single inference request can require billions of floating-point operations, and the memory footprint of these models can exceed 100GB. Traditional horizontal scaling doesn't work well when each node needs such significant resources.

## Our Approach

We developed a hybrid architecture that combines:

1. **Smart Request Routing**: Not all requests are created equal. Short completions can be routed to smaller, faster instances, while complex reasoning tasks go to our most powerful clusters.

2. **Speculative Decoding**: By predicting likely token sequences, we can batch verify multiple tokens at once, dramatically improving throughput.

3. **Dynamic Batching**: Requests are intelligently grouped based on expected completion length, maximizing GPU utilization without sacrificing latency.

4. **Edge Caching**: Common prompts and their variations are cached at edge locations, reducing round-trip time for repeat queries.

## Results

After implementing these optimizations, we achieved:

- **99th percentile latency under 100ms** for standard completions
- **10x improvement in throughput** compared to naive scaling
- **40% reduction in cost per token** through better resource utilization

## Looking Ahead

We're continuing to push the boundaries of what's possible. Our next focus areas include:

- Multi-region inference with automatic failover
- Custom silicon optimization for specific model architectures
- Real-time model switching based on request characteristics

Stay tuned for more technical deep-dives into our infrastructure.
    `,
    author: {
      name: "Sarah Chen",
      role: "Chief Technology Officer",
      avatar: "/professional-woman-headshot.png",
    },
    category: "Engineering",
    publishedAt: "2024-12-15",
    readTime: "8 min read",
    coverImage: "/server-infrastructure-data-center.jpg",
    featured: true,
  },
  {
    slug: "introducing-gpt-4-turbo-support",
    title: "Introducing GPT-4 Turbo Support on Infiner",
    excerpt:
      "We're excited to announce full support for OpenAI's GPT-4 Turbo model with 128K context window and improved performance.",
    content: `
# Introducing GPT-4 Turbo Support on Infiner

Today we're thrilled to announce that GPT-4 Turbo is now available on the Infiner platform. This represents a significant upgrade in capabilities for our users.

## What's New

GPT-4 Turbo brings several improvements:

- **128K Context Window**: Process documents up to ~300 pages in a single request
- **Knowledge Cutoff**: Updated to April 2024
- **Improved Instruction Following**: Better adherence to complex system prompts
- **JSON Mode**: Native support for structured output generation

## How to Use It

Simply update your model parameter to \`gpt-4-turbo\` in your API calls:

\`\`\`javascript
const response = await infiner.chat.completions.create({
  model: "gpt-4-turbo",
  messages: [
    { role: "user", content: "Analyze this document..." }
  ]
});
\`\`\`

## Pricing

GPT-4 Turbo is available at competitive rates:

- Input: $10.00 / 1M tokens
- Output: $30.00 / 1M tokens

This represents significant savings compared to the previous GPT-4 model while delivering improved performance.

## Get Started

GPT-4 Turbo is available now for all Infiner users. Check out our updated documentation for migration guides and best practices.
    `,
    author: {
      name: "Marcus Johnson",
      role: "Product Manager",
      avatar: "/professional-man-headshot.png",
    },
    category: "Product",
    publishedAt: "2024-12-10",
    readTime: "4 min read",
    coverImage: "/ai-neural-network-abstract.jpg",
  },
  {
    slug: "building-rag-applications-best-practices",
    title: "Building RAG Applications: Best Practices for 2024",
    excerpt:
      "A comprehensive guide to building production-ready Retrieval-Augmented Generation applications with modern LLMs.",
    content: `
# Building RAG Applications: Best Practices for 2024

Retrieval-Augmented Generation (RAG) has become the go-to architecture for building LLM applications that need access to private or recent data. Here's what we've learned from helping hundreds of teams deploy RAG systems.

## The Foundation

A solid RAG implementation requires three core components:

1. **Document Processing Pipeline**: How you chunk, embed, and index your documents
2. **Retrieval System**: How you find relevant context for each query
3. **Generation Layer**: How you combine retrieved context with LLM capabilities

## Chunking Strategies

The way you split documents significantly impacts retrieval quality:

- **Semantic Chunking**: Split on topic boundaries, not arbitrary character limits
- **Overlap**: Include 10-20% overlap between chunks to preserve context
- **Metadata**: Attach source, date, and section information to each chunk

## Retrieval Optimization

Beyond basic vector similarity:

- **Hybrid Search**: Combine dense vectors with sparse keyword matching
- **Reranking**: Use a cross-encoder to reorder initial results
- **Query Expansion**: Generate multiple query variants to improve recall

## Prompt Engineering

Structure your prompts for reliability:

\`\`\`
Given the following context:
{retrieved_chunks}

Answer the user's question. If the answer cannot be found in the context, say so clearly.

Question: {user_query}
\`\`\`

## Evaluation

Measure what matters:

- **Retrieval Precision**: Are the retrieved chunks relevant?
- **Answer Faithfulness**: Does the answer stick to the provided context?
- **Answer Relevance**: Does the answer address the user's question?

## Common Pitfalls

Avoid these mistakes:

1. Using chunks that are too large or too small
2. Ignoring metadata in retrieval
3. Not handling "I don't know" cases gracefully
4. Skipping evaluation during development

## Conclusion

RAG systems are powerful but require careful engineering. Start simple, measure everything, and iterate based on real user feedback.
    `,
    author: {
      name: "Emily Zhang",
      role: "Solutions Architect",
      avatar: "/professional-asian-woman-headshot.jpg",
    },
    category: "Tutorial",
    publishedAt: "2024-12-05",
    readTime: "12 min read",
    coverImage: "/documents-and-search-technology.jpg",
  },
  {
    slug: "claude-3-5-sonnet-benchmark-analysis",
    title: "Claude 3.5 Sonnet: A Deep Benchmark Analysis",
    excerpt:
      "We ran extensive benchmarks comparing Claude 3.5 Sonnet against GPT-4 and other leading models. Here are our findings.",
    content: `
# Claude 3.5 Sonnet: A Deep Benchmark Analysis

Anthropic's Claude 3.5 Sonnet has quickly become one of our most requested models. We conducted extensive benchmarks to help our users understand where it excels.

## Methodology

We tested across multiple dimensions:

- **Reasoning**: Complex multi-step problems
- **Coding**: Generation, debugging, and explanation
- **Creative Writing**: Style, coherence, and originality
- **Instruction Following**: Adherence to detailed specifications
- **Speed**: Tokens per second and time to first token

## Key Findings

### Reasoning
Claude 3.5 Sonnet shows exceptional performance on reasoning tasks, matching or exceeding GPT-4 on most benchmarks while being significantly faster.

### Coding
Particularly strong in:
- Python and JavaScript generation
- Bug identification and fixing
- Code explanation and documentation

### Speed
- Average: 1,100 tokens/second
- Time to first token: 12ms average

This makes it one of the fastest frontier models available.

## Recommendations

Based on our analysis:

- **Use Claude 3.5 Sonnet for**: Code generation, document analysis, structured data extraction
- **Consider GPT-4 for**: Tasks requiring the latest knowledge, specific formatting requirements
- **Use Gemini 1.5 Pro for**: Very long context tasks (1M+ tokens)

## Conclusion

Claude 3.5 Sonnet represents an excellent balance of capability and speed. For many use cases, it's now our recommended default model.
    `,
    author: {
      name: "David Park",
      role: "ML Research Lead",
      avatar: "/professional-korean-man-headshot.png",
    },
    category: "Research",
    publishedAt: "2024-11-28",
    readTime: "6 min read",
    coverImage: "/benchmark-charts-analytics.jpg",
  },
  {
    slug: "reducing-ai-costs-without-sacrificing-quality",
    title: "Reducing AI Costs by 60% Without Sacrificing Quality",
    excerpt:
      "Practical strategies for optimizing your LLM spending while maintaining output quality for production applications.",
    content: `
# Reducing AI Costs by 60% Without Sacrificing Quality

AI inference costs can quickly spiral out of control. Here's how our customers are cutting costs dramatically while maintaining quality.

## Strategy 1: Smart Model Selection

Not every task needs GPT-4. Our routing layer automatically selects the right model:

- **Simple Q&A**: Use GPT-3.5 Turbo (90% cheaper)
- **Code generation**: Claude 3.5 Sonnet (better price/performance)
- **Complex reasoning**: GPT-4 only when needed

## Strategy 2: Prompt Optimization

Shorter prompts = lower costs:

- Remove redundant instructions
- Use few-shot examples efficiently
- Leverage system prompts for persistent context

## Strategy 3: Caching

Cache aggressively:

- Semantic caching for similar queries
- Exact match caching for repeated requests
- TTL-based invalidation for time-sensitive data

## Strategy 4: Output Length Control

Constrain outputs appropriately:

- Set max_tokens based on expected response length
- Use structured output (JSON mode) to avoid verbosity
- Implement streaming to cut off when sufficient

## Real Results

One customer reduced their monthly bill from $50,000 to $18,000 by implementing these strategies—a 64% reduction with no measurable quality decrease.

## Getting Started

Our cost optimization dashboard (available to all Pro users) provides personalized recommendations based on your usage patterns.
    `,
    author: {
      name: "Rachel Torres",
      role: "Customer Success Lead",
      avatar: "/professional-latina-woman-headshot.png",
    },
    category: "Guide",
    publishedAt: "2024-11-20",
    readTime: "5 min read",
    coverImage: "/cost-optimization-charts-money.jpg",
  },
  {
    slug: "multimodal-ai-vision-capabilities",
    title: "Unlocking Multimodal AI: Vision Capabilities Explained",
    excerpt:
      "A practical guide to using vision capabilities in modern LLMs for document processing, image analysis, and more.",
    content: `
# Unlocking Multimodal AI: Vision Capabilities Explained

Vision-enabled LLMs open up entirely new application categories. Here's how to make the most of them.

## Supported Models

Currently, these models support vision on Infiner:

- GPT-4 Turbo
- GPT-4o
- Claude 3.5 Sonnet
- Gemini 1.5 Pro

## Use Cases

### Document Processing
Extract structured data from:
- Invoices and receipts
- Forms and applications
- Handwritten notes

### Image Analysis
- Product defect detection
- Medical image analysis
- Real estate photo evaluation

### UI/UX Analysis
- Analyze screenshots for accessibility issues
- Generate code from design mockups
- Compare design implementations

## Implementation

Basic vision request:

\`\`\`javascript
const response = await infiner.chat.completions.create({
  model: "gpt-4o",
  messages: [{
    role: "user",
    content: [
      { type: "text", text: "What's in this image?" },
      { type: "image_url", image_url: { url: "data:image/jpeg;base64,..." } }
    ]
  }]
});
\`\`\`

## Best Practices

1. **Resize images** to reduce costs (max 2048px recommended)
2. **Use specific prompts** for better accuracy
3. **Combine with text context** when available
4. **Handle errors gracefully** for unclear images

## Conclusion

Vision capabilities transform what's possible with AI. Start experimenting today with our interactive playground.
    `,
    author: {
      name: "Alex Rivera",
      role: "Developer Advocate",
      avatar: "/professional-man-headshot-casual.jpg",
    },
    category: "Tutorial",
    publishedAt: "2024-11-15",
    readTime: "7 min read",
    coverImage: "/computer-vision-ai-image-analysis.jpg",
  },
]

export const categories = ["All", "Engineering", "Product", "Tutorial", "Research", "Guide"]

export function getBlogPost(slug: string): BlogPost | undefined {
  return blogPosts.find((post) => post.slug === slug)
}

export function getFeaturedPost(): BlogPost | undefined {
  return blogPosts.find((post) => post.featured)
}
