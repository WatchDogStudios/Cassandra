"use client"

import { useEffect, useRef, useState } from "react"
import Link from "next/link"
import Image from "next/image"
import { blogPosts } from "@/lib/blog-data"
import { Button } from "@/components/ui/button"
import { ArrowRight, Calendar, Clock } from "lucide-react"

export function FeaturedBlogsSection() {
  const [isVisible, setIsVisible] = useState(false)
  const sectionRef = useRef<HTMLElement>(null)

  // Get 3 most recent posts for featured display
  const featuredPosts = blogPosts.slice(0, 3)

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setIsVisible(true)
          observer.disconnect()
        }
      },
      { threshold: 0.2 },
    )

    if (sectionRef.current) {
      observer.observe(sectionRef.current)
    }

    return () => observer.disconnect()
  }, [])

  return (
    <section ref={sectionRef} className="relative py-24 md:py-32 overflow-hidden bg-background">
      {/* Subtle glow */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[100px] rounded-full opacity-[0.06] blur-[80px] pointer-events-none bg-primary" />

      <div className="container mx-auto px-4 md:px-6">
        {/* Header */}
        <div className="flex flex-col md:flex-row md:items-end md:justify-between mb-12 md:mb-16 gap-4">
          <div>
            <h2
              className={`text-3xl md:text-4xl lg:text-5xl font-bold text-foreground mb-4 text-balance ${
                isVisible ? "animate-slide-up-section" : "opacity-0"
              }`}
            >
              Latest from our blog
            </h2>
            <p
              className={`text-muted-foreground text-lg max-w-2xl ${
                isVisible ? "animate-slide-up-section-delayed" : "opacity-0"
              }`}
            >
              Insights, tutorials, and updates from the Infiner team.
            </p>
          </div>
          <div
            className={`${isVisible ? "animate-slide-up-section-delayed" : "opacity-0"}`}
            style={{ animationDelay: "0.2s" }}
          >
            <Button variant="outline" asChild className="group bg-transparent">
              <Link href="/blog">
                View all articles
                <ArrowRight className="ml-2 h-4 w-4 transition-transform group-hover:translate-x-1" />
              </Link>
            </Button>
          </div>
        </div>

        {/* Blog Cards Grid */}
        <div
          className={`grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 ${
            isVisible ? "animate-slide-up-section-delayed" : "opacity-0"
          }`}
          style={{ animationDelay: "0.3s" }}
        >
          {featuredPosts.map((post, index) => (
            <Link
              key={post.slug}
              href={`/blog/${post.slug}`}
              className="group block"
              style={{ animationDelay: `${0.3 + index * 0.1}s` }}
            >
              <article className="h-full rounded-xl border border-border/50 bg-surface-elevated overflow-hidden transition-all duration-300 hover:border-primary/50 hover:shadow-lg hover:shadow-primary/5">
                {/* Cover Image */}
                <div className="relative aspect-[16/10] overflow-hidden">
                  <Image
                    src={post.coverImage || "/placeholder.svg"}
                    alt={post.title}
                    fill
                    loading="lazy"
                    className="object-cover transition-transform duration-500 group-hover:scale-105"
                  />
                  <div className="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent" />
                  <div className="absolute bottom-3 left-3">
                    <span className="px-2.5 py-1 text-xs font-medium rounded-full bg-primary/90 text-primary-foreground">
                      {post.category}
                    </span>
                  </div>
                </div>

                {/* Content */}
                <div className="p-5">
                  <h3 className="text-lg font-semibold text-foreground mb-2 line-clamp-2 group-hover:text-primary transition-colors">
                    {post.title}
                  </h3>
                  <p className="text-muted-foreground text-sm line-clamp-2 mb-4">{post.excerpt}</p>

                  {/* Meta */}
                  <div className="flex items-center gap-4 text-xs text-muted-foreground">
                    <div className="flex items-center gap-1.5">
                      <Calendar className="h-3.5 w-3.5" />
                      <span>
                        {new Date(post.publishedAt).toLocaleDateString("en-US", {
                          month: "short",
                          day: "numeric",
                          year: "numeric",
                        })}
                      </span>
                    </div>
                    <div className="flex items-center gap-1.5">
                      <Clock className="h-3.5 w-3.5" />
                      <span>{post.readTime}</span>
                    </div>
                  </div>
                </div>
              </article>
            </Link>
          ))}
        </div>
      </div>
    </section>
  )
}
