# Infiner - AI Inference Platform Template

A premium, production-ready marketing website template for AI/ML inference platforms. Built with Next.js 15, Tailwind CSS v4, and modern React patterns.

![Infiner Template](https://blob.v0.app/infiner-hero.png)

## Why This Template?

- **Production-Ready** - 14 fully designed pages with responsive layouts
- **Modern Stack** - Next.js 15, Tailwind CSS v4, TypeScript
- **Dark/Light Mode** - Full theme support with system preference detection
- **Smooth Animations** - Lenis smooth scroll, staggered entrances, scroll-triggered reveals
- **Mobile-First** - Responsive design with a stunning full-screen mobile menu
- **SEO Optimized** - Proper metadata, semantic HTML, accessible components
- **Design System** - Tokenized colors, typography, and spacing for easy customization
- **Developer Experience** - Clean code, consistent patterns, comprehensive style guide

## Features

### Marketing Pages
- Animated hero with staggered text entrance
- Logo marquee (trusted by section)
- Bento grid features with animated beams & 3D globe
- Streaming text animation for model metrics
- Pricing cards with feature comparison
- Testimonials wall with infinite scroll
- Accordion FAQ section
- Newsletter signup with email capture

### Application Pages
- API Playground with mock streaming responses
- Model comparison with filtering & search
- Comprehensive documentation with copy-as-markdown
- Blog system with categories and individual post pages
- Authentication flows (login, signup, forgot password)
- Contact/demo request form

### Utility Pages
- About page with animated team member cards
- Terms of Service
- Privacy Policy
- Style Guide (design system reference)

## Active Routes

| Route | Description |
|-------|-------------|
| `/` | Homepage with all marketing sections |
| `/about` | Team members, company values, stats |
| `/blog` | Blog listing with category filters |
| `/blog/[slug]` | Individual blog post with markdown rendering |
| `/contact` | Enterprise demo request form |
| `/docs` | API documentation with syntax highlighting |
| `/forgot-password` | Password reset flow |
| `/login` | Sign in page |
| `/signup` | Sign up page |
| `/models` | Model pricing & comparison table |
| `/playground` | Interactive API sandbox |
| `/privacy` | Privacy policy |
| `/terms` | Terms of service |
| `/style-guide` | Design system reference |

## Tech Stack

| Category | Technology |
|----------|------------|
| Framework | Next.js 15 (App Router) |
| Styling | Tailwind CSS v4 |
| UI Components | shadcn/ui |
| Animations | CSS keyframes + Intersection Observer |
| Smooth Scroll | Lenis |
| Icons | Lucide React |
| 3D Globe | Cobe |
| Theme | next-themes |
| Syntax Highlighting | Custom brand-colored highlighter |

## Getting Started

### Installation

```bash
# Clone the template
npx shadcn@latest init

# Install dependencies (choose one)
pnpm install   # recommended
npm install
yarn install

# Start development server
pnpm dev   # or npm run dev / yarn dev
```

### Quick Start

1. Open `http://localhost:3000` to see the template
2. Visit `/style-guide` to explore the design system
3. Customize brand colors in `app/globals.css`
4. Update content in component files

## Making It Production-Ready

### 1. Branding

**Update Logo:**
- Replace `public/cass-logo-small.svg` with your logo
- Update logo references in `components/navbar.tsx` and `components/footer.tsx`

**Change Brand Colors:**
```css
/* app/globals.css */
:root {
  --primary: #YOUR_BRAND_COLOR;
}
```

**Update Metadata:**
```tsx
/* app/layout.tsx */
export const metadata: Metadata = {
  title: "Your Company Name",
  description: "Your company description",
}
```

### 2. Content

**Homepage Sections:**
- `components/hero-section.tsx` - Hero headline and CTA
- `components/features-section.tsx` - Feature cards
- `components/pricing-section.tsx` - Pricing tiers
- `components/faq-section.tsx` - FAQ items
- `components/testimonials-section.tsx` - Customer quotes

**Blog Posts:**
- Update `lib/blog-data.ts` with your articles
- Add author images to `public/`

**Team Members:**
- Update `app/about/page.tsx` with your team
- Add headshot images to `public/`

**Models/Products:**
- Update `app/models/page.tsx` with your offerings
- Customize filters and pricing

### 3. Authentication (Optional)

The template includes UI-only auth pages that work standalone for demos. These are purely frontend components with no backend functionality - perfect for prototyping or as a starting point for your own auth implementation.

### 4. Forms & Data

**Contact Form:**
- Add form submission handler in `app/contact/page.tsx`
- Integrate with email service (Resend, SendGrid, etc.)

**Newsletter:**
- Update `components/footer.tsx` email form
- Connect to email marketing (Mailchimp, ConvertKit, etc.)

**Blog Content:**
- Update blog posts in `lib/blog-data.ts`
- For dynamic content, consider a headless CMS (Sanity, Contentful, Notion)

### 5. Analytics & Monitoring

**Vercel Integrations (Pre-configured):**

This template includes commented-out code for Vercel integrations. To enable:

1. **Analytics** - Uncomment in `app/layout.tsx`:
```tsx
import { Analytics } from '@vercel/analytics/react'
// Add <Analytics /> before closing body tag
```

2. **Speed Insights** - Uncomment in `app/layout.tsx`:
```tsx
import { SpeedInsights } from '@vercel/speed-insights/next'
// Add <SpeedInsights /> before closing body tag
```

3. **Open Graph Images** - Uncomment `/api/og/route.tsx` to enable dynamic OG image generation

**Other Analytics Options:**
```bash
# PostHog
npm install posthog-js

# Google Analytics
npm install @next/third-parties
```

### 6. Deployment

**Vercel (Recommended):**
```bash
npm install -g vercel
vercel
```

**Environment Variables:**
Add any required env vars in Vercel dashboard or `.env.local`:
```
DATABASE_URL=
NEXT_PUBLIC_SITE_URL=
RESEND_API_KEY=
```

## Project Structure

```
app/
├── globals.css              # Design tokens, animations
├── layout.tsx               # Root layout with providers
├── page.tsx                 # Homepage
├── about/page.tsx           # About & team
├── blog/
│   ├── page.tsx             # Blog listing
│   └── [slug]/page.tsx      # Blog post
├── contact/page.tsx         # Contact form
├── docs/page.tsx            # Documentation
├── forgot-password/page.tsx # Password reset
├── login/page.tsx           # Sign in
├── signup/page.tsx          # Sign up
├── models/page.tsx          # Model comparison
├── playground/page.tsx      # API sandbox
├── privacy/page.tsx         # Privacy policy
├── terms/page.tsx           # Terms of service
└── style-guide/page.tsx     # Design system

components/
├── navbar.tsx               # Navigation with mobile menu
├── footer.tsx               # Footer with newsletter
├── hero-section.tsx         # Homepage hero
├── features-section.tsx     # Bento grid features
├── pricing-section.tsx      # Pricing cards
├── testimonials-section.tsx # Testimonials wall
├── faq-section.tsx          # FAQ accordion
├── featured-blogs-section.tsx # Blog preview
├── logo-marquee.tsx         # Logo carousel
├── models-table-section.tsx # Model metrics table
├── grid-background.tsx      # Decorative grid
├── theme-provider.tsx       # Theme context
├── theme-switcher.tsx       # Theme toggle
├── lenis-provider.tsx       # Smooth scroll
├── scroll-to-top.tsx        # Page scroll reset
├── docs/                    # Documentation components
├── magicui/                 # Animated components
└── ui/                      # shadcn/ui components

lib/
├── blog-data.ts             # Blog content
├── docs-content.ts          # Documentation content
├── syntax-highlighter.tsx   # Code highlighting
└── utils.ts                 # Utility functions
```

## Design System

### Colors

| Token | Light | Dark | Usage |
|-------|-------|------|-------|
| `--background` | `#ffffff` | `#0a0e17` | Page background |
| `--foreground` | `#0a0e17` | `#ffffff` | Primary text |
| `--primary` | `#225EDF` | `#225EDF` | Brand color, CTAs |
| `--muted` | `#f1f5f9` | `#1e293b` | Subtle backgrounds |
| `--muted-foreground` | `#64748b` | `#94a3b8` | Secondary text |
| `--card` | `#ffffff` | `#0f1629` | Card backgrounds |
| `--border` | `#e2e8f0` | `#1e293b` | Borders |

### Typography

- **Font**: Geist Sans (headings & body), Geist Mono (code)
- **Scale**: text-sm → text-7xl
- **Line Height**: leading-relaxed for body copy

### Animations

| Animation | Description |
|-----------|-------------|
| `slide-up` | Entrance from bottom |
| `slide-in-right` | Entrance from right |
| `slide-in-left` | Entrance from left |
| `fade-in` | Opacity fade |
| `marquee` | Infinite horizontal scroll |
| `scroll-up/down` | Infinite vertical scroll |
| `pulse-slow` | Gentle pulsing |
| `shimmer` | Loading shimmer effect |

## Browser Support

- Chrome, Firefox, Safari, Edge (latest)
- Custom scrollbar styling
- CSS animations with GPU acceleration
- Responsive from 320px to 4K

## Support & Community

Need help with the template? Have questions or suggestions?

- **Vercel Community**: [community.vercel.com](https://community.vercel.com)
- **Twitter/X**: [@Webrenew_](https://twitter.com/Webrenew_)

We're here to help you build something amazing!

## License

MIT License - Free for personal and commercial use.

---

Built with Next.js, Tailwind CSS, and shadcn/ui.
