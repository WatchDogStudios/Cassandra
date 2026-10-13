"use client"

import { useEffect, useRef, useState } from "react"

const testimonials = [
  {
    body: "Switched from OpenAI direct to Infiner and our latency dropped by 40%. The model routing is incredibly smart.",
    author: {
      name: "Sarah Chen",
      handle: "sarahchen_dev",
      role: "CTO at Stackwise",
      imageUrl: "/professional-asian-woman-headshot.jpg",
    },
  },
  {
    body: "Finally an inference platform that just works. No more juggling API keys across providers.",
    author: {
      name: "Marcus Rodriguez",
      handle: "marcusdev",
      role: "Founder at AIFlow",
      imageUrl: "/professional-latino-man-headshot.png",
    },
  },
  {
    body: "The auto-scaling saved us during our Product Hunt launch. Went from 100 to 10,000 requests/min seamlessly.",
    author: {
      name: "Emily Watson",
      handle: "emwatson",
      role: "Engineering Lead at Prompt.io",
      imageUrl: "/professional-blonde-woman-headshot.png",
    },
  },
  {
    body: "We've cut our AI infrastructure costs by 60% with Infiner's intelligent routing. Game changer.",
    author: {
      name: "David Park",
      handle: "davidpark_ai",
      role: "VP Engineering at Nexus",
      imageUrl: "/professional-korean-man-headshot.png",
    },
  },
  {
    body: "Best developer experience I've seen in the AI space. SDK is clean, docs are excellent.",
    author: {
      name: "Priya Sharma",
      handle: "priyacodes",
      role: "Senior Engineer at CloudScale",
      imageUrl: "/professional-indian-woman-headshot.png",
    },
  },
  {
    body: "12ms average latency is insane. Our chatbot feels instant now.",
    author: {
      name: "James Liu",
      handle: "jamesliu",
      role: "Founder at ChatBot.ai",
      imageUrl: "/professional-chinese-man-headshot.jpg",
    },
  },
  {
    body: "Migrated our entire stack to Infiner in a day. Zero downtime, immediate performance gains.",
    author: {
      name: "Anna Kowalski",
      handle: "annak_tech",
      role: "Platform Lead at Veritas",
      imageUrl: "/professional-eastern-european-woman-headshot.jpg",
    },
  },
  {
    body: "The enterprise support is phenomenal. Had a custom integration running within hours.",
    author: {
      name: "Michael Foster",
      handle: "michaelfoster",
      role: "CTO at DataSync",
      imageUrl: "/professional-man-suit-headshot.png",
    },
  },
  {
    body: "99.99% uptime isn't marketing speak with Infiner. We've had zero incidents in 6 months.",
    author: {
      name: "Rachel Green",
      handle: "rachelg_dev",
      role: "SRE at FinanceAI",
      imageUrl: "/professional-redhead-woman-headshot.jpg",
    },
  },
]

function TestimonialCard({ testimonial }: { testimonial: (typeof testimonials)[0] }) {
  return (
    <figure className="rounded-xl bg-surface-testimonial border border-primary/10 p-6 mb-4">
      <blockquote className="text-muted-foreground text-sm leading-relaxed">
        <p>{`"${testimonial.body}"`}</p>
      </blockquote>
      <figcaption className="mt-4 flex items-center gap-x-3">
        <img
          alt=""
          src={testimonial.author.imageUrl || "/placeholder.svg"}
          width={40}
          height={40}
          loading="lazy"
          className="size-10 rounded-full bg-muted"
        />
        <div>
          <div className="font-medium text-foreground text-sm">{testimonial.author.name}</div>
          <div className="text-muted-foreground text-xs">@{testimonial.author.handle}</div>
        </div>
      </figcaption>
    </figure>
  )
}

function ScrollingColumn({
  testimonials,
  direction = "up",
  duration = 25,
}: {
  testimonials: (typeof testimonials)[0][]
  direction?: "up" | "down"
  duration?: number
}) {
  return (
    <div className="relative h-[600px] overflow-hidden">
      <div className="absolute top-0 left-0 right-0 h-32 bg-gradient-to-b from-surface-dark to-transparent z-10 pointer-events-none" />
      <div className="absolute bottom-0 left-0 right-0 h-32 bg-gradient-to-t from-surface-dark to-transparent z-10 pointer-events-none" />

      <div
        className={`flex flex-col ${direction === "up" ? "animate-scroll-up" : "animate-scroll-down"}`}
        style={{
          animationDuration: `${duration}s`,
        }}
      >
        {[...testimonials, ...testimonials].map((testimonial, idx) => (
          <TestimonialCard key={`${testimonial.author.handle}-${idx}`} testimonial={testimonial} />
        ))}
      </div>
    </div>
  )
}

export function TestimonialsSection() {
  const sectionRef = useRef<HTMLElement>(null)
  const [isVisible, setIsVisible] = useState(false)

  useEffect(() => {
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setIsVisible(true)
          observer.disconnect()
        }
      },
      { threshold: 0.1 },
    )

    if (sectionRef.current) {
      observer.observe(sectionRef.current)
    }

    return () => observer.disconnect()
  }, [])

  const col1 = testimonials.slice(0, 3)
  const col2 = testimonials.slice(3, 6)
  const col3 = testimonials.slice(6, 9)
  const allTestimonials = testimonials

  return (
    <section ref={sectionRef} className="relative py-24 md:py-32 overflow-hidden bg-surface-dark">
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[1000px] h-[200px] rounded-full pointer-events-none bg-primary/[0.08] blur-[120px]" />

      <div className="w-full flex justify-center px-4">
        <div className="max-w-5xl w-full">
          <div className={`text-center mb-16 ${isVisible ? "animate-slide-up-section" : "opacity-0"}`}>
            <p className="text-primary text-sm font-medium mb-3">Wall of Love</p>
            <h2 className="text-3xl md:text-4xl lg:text-5xl font-bold text-foreground text-balance mb-4">
              Loved by developers worldwide
            </h2>
            <p
              className={`text-muted-foreground max-w-2xl mx-auto ${isVisible ? "animate-slide-up-section-delayed" : "opacity-0"}`}
            >
              Join thousands of teams building the future of AI with Infiner.
            </p>
          </div>

          <div className={`md:hidden ${isVisible ? "animate-fade-in-delayed" : "opacity-0"}`}>
            <ScrollingColumn testimonials={allTestimonials} direction="up" duration={45} />
          </div>

          <div className={`hidden md:grid md:grid-cols-3 gap-4 ${isVisible ? "animate-fade-in-delayed" : "opacity-0"}`}>
            <ScrollingColumn testimonials={col1} direction="up" duration={30} />
            <ScrollingColumn testimonials={col2} direction="down" duration={35} />
            <ScrollingColumn testimonials={col3} direction="up" duration={32} />
          </div>
        </div>
      </div>
    </section>
  )
}
