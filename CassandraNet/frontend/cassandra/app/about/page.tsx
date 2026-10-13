"use client"

import { useState } from "react"
import Link from "next/link"
import Image from "next/image"
import { Navbar } from "@/components/navbar"
import { Footer } from "@/components/footer"
import { GridBackground } from "@/components/grid-background"
import { ScrollToTop } from "@/components/scroll-to-top"
import { Button } from "@/components/ui/button"
import { Twitter, Linkedin, Github, ArrowRight, Sparkles, Target, Users, Zap } from "lucide-react"

const teamMembers = [
  {
    name: "Sarah Chen",
    role: "Co-Founder & CEO",
    bio: "Former ML Lead at Google Brain. PhD in Computer Science from Stanford.",
    image: "/professional-asian-woman-ceo-headshot.jpg",
    social: {
      twitter: "#",
      linkedin: "#",
      github: "#",
    },
  },
  {
    name: "Marcus Johnson",
    role: "Co-Founder & CTO",
    bio: "Ex-Principal Engineer at OpenAI. 15+ years in distributed systems.",
    image: "/professional-black-cto.png",
    social: {
      twitter: "#",
      linkedin: "#",
      github: "#",
    },
  },
  {
    name: "Elena Rodriguez",
    role: "VP of Engineering",
    bio: "Previously built ML infrastructure at Meta. Expert in GPU optimization.",
    image: "/professional-latina-woman-engineer-headshot.jpg",
    social: {
      twitter: "#",
      linkedin: "#",
      github: "#",
    },
  },
  {
    name: "David Park",
    role: "Head of Product",
    bio: "Former Product Lead at Anthropic. Passionate about developer experience.",
    image: "/professional-korean-man-product-manager-headshot.jpg",
    social: {
      twitter: "#",
      linkedin: "#",
      github: "#",
    },
  },
  {
    name: "Priya Sharma",
    role: "Head of Research",
    bio: "PhD from MIT. Published 30+ papers on large language models.",
    image: "/professional-indian-woman-researcher-headshot.jpg",
    social: {
      twitter: "#",
      linkedin: "#",
    },
  },
  {
    name: "James Mitchell",
    role: "VP of Sales",
    bio: "Former Enterprise Sales Director at AWS. Built $100M+ revenue teams.",
    image: "/professional-man-sales-executive-headshot.jpg",
    social: {
      twitter: "#",
      linkedin: "#",
    },
  },
]

const values = [
  {
    icon: Zap,
    title: "Speed Obsessed",
    description: "We believe every millisecond matters. Our infrastructure is built for the lowest latency possible.",
  },
  {
    icon: Target,
    title: "Customer First",
    description: "Every decision we make starts with our customers. Their success is our success.",
  },
  {
    icon: Users,
    title: "Open & Transparent",
    description: "We believe in open communication, transparent pricing, and honest relationships.",
  },
  {
    icon: Sparkles,
    title: "Innovation Driven",
    description: "We push the boundaries of what's possible in AI infrastructure every single day.",
  },
]

const stats = [
  { value: "500+", label: "Enterprise Customers" },
  { value: "99.99%", label: "Uptime SLA" },
  { value: "50B+", label: "API Calls / Month" },
  { value: "< 50ms", label: "Average Latency" },
]

function TeamCard({ member, index }: { member: (typeof teamMembers)[0]; index: number }) {
  const [isHovered, setIsHovered] = useState(false)

  return (
    <div className="group relative" onMouseEnter={() => setIsHovered(true)} onMouseLeave={() => setIsHovered(false)}>
      {/* Card container */}
      <div className="relative overflow-hidden rounded-2xl bg-card border border-border transition-all duration-500 hover:border-primary/50 hover:shadow-[0_0_40px_-10px_rgba(34,94,223,0.3)]">
        {/* Image container */}
        <div className="relative aspect-[3/4] md:aspect-[4/5] overflow-hidden">
          <Image
            src={member.image || "/placeholder.svg"}
            alt={member.name}
            fill
            loading="lazy"
            className="object-cover object-top transition-all duration-700 grayscale group-hover:grayscale-0 group-hover:scale-110"
          />

          {/* Gradient overlay */}
          <div className="absolute inset-0 bg-gradient-to-t from-card via-card/40 to-transparent opacity-80 group-hover:opacity-90 transition-opacity duration-500" />
        </div>

        {/* Content */}
        <div className="absolute bottom-0 left-0 right-0 p-4 md:p-6">
          {/* Role badge */}
          <div className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-primary/10 border border-primary/20 mb-2">
            <span className="w-1.5 h-1.5 rounded-full bg-primary animate-pulse" />
            <span className="text-xs font-medium text-primary">{member.role}</span>
          </div>

          {/* Name */}
          <h3 className="text-lg md:text-xl font-semibold text-foreground mb-1">{member.name}</h3>

          {/* Bio - slides up on hover */}
          <p
            className={`text-sm text-muted-foreground transition-all duration-500 ${
              isHovered ? "opacity-100 translate-y-0 max-h-20" : "opacity-0 translate-y-4 max-h-0"
            } overflow-hidden`}
          >
            {member.bio}
          </p>

          {/* Social links - slide up on hover */}
          <div
            className={`flex items-center gap-2 mt-3 transition-all duration-500 delay-100 ${
              isHovered ? "opacity-100 translate-y-0 max-h-12" : "opacity-0 translate-y-4 max-h-0"
            } overflow-hidden`}
          >
            {member.social.twitter && (
              <Link
                href={member.social.twitter}
                target="_blank"
                rel="noopener noreferrer"
                className="p-2 rounded-lg bg-background/50 border border-border text-muted-foreground hover:text-primary hover:border-primary/50 hover:bg-primary/10 transition-all"
                aria-label={`${member.name} on Twitter`}
              >
                <Twitter className="w-4 h-4" />
              </Link>
            )}
            {member.social.linkedin && (
              <Link
                href={member.social.linkedin}
                target="_blank"
                rel="noopener noreferrer"
                className="p-2 rounded-lg bg-background/50 border border-border text-muted-foreground hover:text-primary hover:border-primary/50 hover:bg-primary/10 transition-all"
                aria-label={`${member.name} on LinkedIn`}
              >
                <Linkedin className="w-4 h-4" />
              </Link>
            )}
            {member.social.github && (
              <Link
                href={member.social.github}
                target="_blank"
                rel="noopener noreferrer"
                className="p-2 rounded-lg bg-background/50 border border-border text-muted-foreground hover:text-primary hover:border-primary/50 hover:bg-primary/10 transition-all"
                aria-label={`${member.name} on GitHub`}
              >
                <Github className="w-4 h-4" />
              </Link>
            )}
          </div>
        </div>

        {/* Corner accent */}
        <div className="absolute top-4 right-4 w-8 h-8 opacity-0 group-hover:opacity-100 transition-opacity duration-500">
          <div className="absolute top-0 right-0 w-full h-[2px] bg-gradient-to-l from-primary to-transparent" />
          <div className="absolute top-0 right-0 h-full w-[2px] bg-gradient-to-b from-primary to-transparent" />
        </div>
      </div>
    </div>
  )
}

export default function AboutPage() {
  return (
    <div className="min-h-screen bg-background">
      <ScrollToTop />
      <GridBackground />
      <Navbar />

      {/* Custom hero gradient */}
      <div
        className="pointer-events-none absolute inset-x-0 top-0 h-[600px] overflow-hidden"
        style={{
          background: "radial-gradient(circle 800px at 50% -200px, rgba(34, 94, 223, 0.15), transparent 70%)",
        }}
      />

      <main className="relative z-10 pt-32 pb-20">
        {/* Hero section */}
        <section className="container mx-auto px-6 text-center mb-32 md:mb-24">
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-primary/10 border border-primary/20 mb-6">
            <Sparkles className="w-4 h-4 text-primary" />
            <span className="text-sm font-medium text-primary">About Infiner</span>
          </div>

          <h1 className="text-4xl md:text-5xl lg:text-6xl font-bold text-foreground mb-6 max-w-4xl mx-auto text-balance">
            Building the future of <span className="text-primary">AI infrastructure</span>
          </h1>

          <p className="text-lg text-muted-foreground max-w-2xl mx-auto mb-10">
            We're a team of engineers, researchers, and builders on a mission to make AI inference fast, reliable, and
            accessible to everyone.
          </p>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-4">
            <Button asChild size="lg" className="bg-primary hover:bg-primary/90">
              <Link href="/contact">
                Get in Touch
                <ArrowRight className="w-4 h-4 ml-2" />
              </Link>
            </Button>
            <Button asChild variant="outline" size="lg">
              <Link href="#team">Meet the Team</Link>
            </Button>
          </div>
        </section>

        {/* Stats section */}
        <section className="container mx-auto px-6 mb-24">
          <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-4 gap-4">
            {stats.map((stat, index) => (
              <div
                key={index}
                className="relative p-6 rounded-xl bg-card/50 border border-border text-center group hover:border-primary/30 transition-colors"
              >
                <div className="text-3xl md:text-4xl font-bold text-primary mb-2">{stat.value}</div>
                <div className="text-sm text-muted-foreground">{stat.label}</div>
              </div>
            ))}
          </div>
        </section>

        {/* Mission section */}
        <section className="container mx-auto px-6 mb-24">
          <div className="max-w-4xl mx-auto">
            <div className="relative p-8 md:p-12 rounded-2xl bg-card/50 border border-border overflow-hidden">
              {/* Decorative gradient */}
              <div className="absolute top-0 right-0 w-64 h-64 bg-primary/5 rounded-full blur-3xl" />

              <div className="relative">
                <h2 className="text-2xl md:text-3xl font-bold text-foreground mb-6">Our Mission</h2>
                <p className="text-lg text-muted-foreground leading-relaxed mb-6">
                  We founded Infiner with a simple belief: AI should be accessible to every developer and every company,
                  regardless of scale. The infrastructure that powers AI shouldn't be a bottleneck—it should be an
                  enabler.
                </p>
                <p className="text-lg text-muted-foreground leading-relaxed">
                  Today, we're building the most reliable, fastest, and most cost-effective AI inference platform in the
                  world. We're not just building technology—we're building the foundation for the next generation of
                  AI-powered applications.
                </p>
              </div>
            </div>
          </div>
        </section>

        {/* Values section */}
        <section className="container mx-auto px-6 mb-24">
          <div className="text-center mb-12">
            <h2 className="text-3xl md:text-4xl font-bold text-foreground mb-4">Our Values</h2>
            <p className="text-muted-foreground max-w-2xl mx-auto">The principles that guide everything we do</p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {values.map((value, index) => (
              <div
                key={index}
                className="group p-6 rounded-xl bg-card/50 border border-border hover:border-primary/30 transition-all duration-300"
              >
                <div className="w-12 h-12 rounded-xl bg-primary/10 flex items-center justify-center mb-4 group-hover:scale-110 transition-transform">
                  <value.icon className="w-6 h-6 text-primary" />
                </div>
                <h3 className="text-lg font-semibold text-foreground mb-2">{value.title}</h3>
                <p className="text-sm text-muted-foreground">{value.description}</p>
              </div>
            ))}
          </div>
        </section>

        {/* Team section */}
        <section id="team" className="container mx-auto px-6 mb-24">
          <div className="text-center mb-12">
            <h2 className="text-3xl md:text-4xl font-bold text-foreground mb-4">Meet Our Team</h2>
            <p className="text-muted-foreground max-w-2xl mx-auto">
              World-class talent from the best AI and infrastructure companies
            </p>
          </div>

          <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            {teamMembers.map((member, index) => (
              <TeamCard key={member.name} member={member} index={index} />
            ))}
          </div>
        </section>

        {/* CTA section */}
        <section className="container mx-auto px-6">
          <div className="relative p-8 md:p-12 rounded-2xl bg-gradient-to-br from-primary/10 to-primary/5 border border-primary/20 text-center overflow-hidden">
            <div className="absolute inset-0 bg-[radial-gradient(circle_at_30%_50%,rgba(34,94,223,0.1),transparent_50%)]" />

            <div className="relative">
              <h2 className="text-2xl md:text-3xl font-bold text-foreground mb-4">Join Our Team</h2>
              <p className="text-muted-foreground max-w-xl mx-auto mb-8">
                We're always looking for talented people who share our passion for building exceptional AI
                infrastructure. Check out our open positions.
              </p>
              <Button asChild size="lg" className="bg-primary hover:bg-primary/90">
                <Link href="#">
                  View Open Positions
                  <ArrowRight className="w-4 h-4 ml-2" />
                </Link>
              </Button>
            </div>
          </div>
        </section>
      </main>

      <Footer />
    </div>
  )
}
