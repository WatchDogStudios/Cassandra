# Customization Guide

This guide walks you through customizing the Infiner template for your own brand and product.

## Quick Start (5 minutes)

### 1. Update Branding

**Logo**: Replace the logo in `public/cass-logo-small.svg` with your own SVG logo.

**Favicon**: Replace `public/icon.svg` with your brand icon.

**Site Name**: Update the metadata in `app/layout.tsx`:

```tsx
export const metadata: Metadata = {
  title: "Your Brand - Your Tagline",
  description: "Your product description.",
}
```

### 2. Update Colors

Edit `app/globals.css` to change the color scheme. The key tokens are:

```css
@theme inline {
  /* Primary brand color */
  --color-primary: oklch(54.51% 0.2085 262.89);
  
  /* Background colors */
  --color-background: #ffffff;          /* Light mode */
  --color-background: #080b13;          /* Dark mode */
  
  /* Accent/chart colors */
  --color-chart-1: oklch(54.51% 0.2085 262.89);
}
```

**Color Selection Tips:**
- Choose 1 primary brand color
- Use 2-3 neutrals (white, grays, black variants)
- Add 1-2 accent colors for charts/highlights

### 3. Update Fonts

Edit `app/layout.tsx` to change the font:

```tsx
import { Inter } from "next/font/google"
const inter = Inter({ subsets: ["latin"] })
```

Then update `app/globals.css`:

```css
@theme inline {
  --font-sans: 'Inter', 'Inter Fallback';
}
```

---

## Detailed Customization

### Navigation

Edit `components/navbar.tsx` to update navigation links:

```tsx
const navItems = [
  { label: "Your Link", href: "/your-path" },
  // Add more items...
]
```

### Footer

Edit `components/footer.tsx` to update:
- Company information
- Link sections
- Social media links

### Homepage Sections

Each section is a separate component in `/components`:

| Section | File | Purpose |
|---------|------|---------|
| Hero | `hero-section.tsx` | Main headline and CTA |
| Features | `features-section.tsx` | Product features grid |
| Models Table | `models-table-section.tsx` | Pricing/comparison table |
| Pricing | `pricing-section.tsx` | Pricing cards |
| Testimonials | `testimonials-section.tsx` | Customer quotes |
| FAQ | `faq-section.tsx` | Common questions |
| Featured Blogs | `featured-blogs-section.tsx` | Blog previews |

### Blog Content

Edit `lib/blog-data.ts` to add/modify blog posts:

```tsx
export const blogPosts: BlogPost[] = [
  {
    slug: "your-post-slug",
    title: "Your Post Title",
    excerpt: "Brief description...",
    content: `Your markdown content here...`,
    category: "Product",
    author: { name: "Author Name", ... },
    // ...
  },
]
```

### Team Members

Edit `app/about/page.tsx` to update the team section:

```tsx
const teamMembers = [
  {
    name: "Team Member",
    role: "Their Role",
    bio: "Short bio...",
    image: "/path-to-image.jpg",
    socials: { twitter: "...", linkedin: "..." },
  },
]
```

---

## Adding New Pages

1. Create a new folder in `/app` with your route name
2. Add a `page.tsx` file for the content
3. Add a `layout.tsx` file for SEO metadata
4. Update navigation in `components/navbar.tsx`
5. Update footer links in `components/footer.tsx`

Example structure:
```
app/
  your-page/
    page.tsx      # Page content
    layout.tsx    # SEO metadata
```

---

## Design Tokens Reference

### Colors

| Token | Usage |
|-------|-------|
| `--background` | Page background |
| `--foreground` | Primary text |
| `--muted` | Subtle backgrounds |
| `--muted-foreground` | Secondary text |
| `--primary` | Brand color, buttons |
| `--primary-foreground` | Text on primary |
| `--secondary` | Secondary buttons |
| `--accent` | Highlights |
| `--border` | Borders, dividers |
| `--ring` | Focus rings |

### Spacing Scale

Use Tailwind's spacing scale: `p-4`, `m-6`, `gap-8`, etc.

Avoid arbitrary values like `p-[17px]`.

### Typography Scale

| Class | Size | Usage |
|-------|------|-------|
| `text-xs` | 12px | Captions |
| `text-sm` | 14px | Secondary text |
| `text-base` | 16px | Body text |
| `text-lg` | 18px | Lead text |
| `text-xl` | 20px | Small headings |
| `text-2xl` | 24px | Section headings |
| `text-3xl` | 30px | Page headings |
| `text-4xl+` | 36px+ | Hero headings |

---

## Component Library

This template uses [shadcn/ui](https://ui.shadcn.com/) components. All components are in `/components/ui/`.

To add new shadcn components:

```bash
npx shadcn@latest add [component-name]
```

Available components: accordion, alert, avatar, badge, button, card, checkbox, dialog, dropdown-menu, input, label, select, switch, tabs, textarea, tooltip, and more.

---

## Animations

Custom animations are defined in `app/globals.css`:

- `animate-fade-in` - Fade in
- `animate-slide-up` - Slide up and fade
- `animate-slide-in-left` - Slide from left
- `animate-slide-in-right` - Slide from right
- `animate-scale-in` - Scale up and fade

Use with intersection observer for scroll-triggered animations.

---

## Need Help?

- Twitter/X: [@Webrenew_](https://twitter.com/Webrenew_)
- Vercel Community: [community.vercel.com](https://vercel.com/community)
