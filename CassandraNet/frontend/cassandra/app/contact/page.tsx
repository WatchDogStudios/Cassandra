"use client"

import type React from "react"

import { useState } from "react"
import Link from "next/link"
import Image from "next/image"
import { Button } from "@/components/ui/button"
import { GridBackground } from "@/components/grid-background"
import { Calendar, Clock, Video, CheckCircle2, Building2, Users, Globe } from "lucide-react"

const timeSlots = ["9:00 AM", "10:00 AM", "11:00 AM", "1:00 PM", "2:00 PM", "3:00 PM", "4:00 PM"]

const benefits = [
  { icon: Building2, text: "Custom deployment options" },
  { icon: Users, text: "Dedicated support team" },
  { icon: Globe, text: "Global infrastructure" },
]

export default function ContactPage() {
  const [formData, setFormData] = useState({
    firstName: "",
    lastName: "",
    email: "",
    company: "",
    jobTitle: "",
    teamSize: "",
    message: "",
  })
  const [selectedDate, setSelectedDate] = useState<number | null>(null)
  const [selectedTime, setSelectedTime] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitted, setIsSubmitted] = useState(false)

  const today = new Date()
  const currentMonth = today.toLocaleString("default", { month: "long" })
  const currentYear = today.getFullYear()
  const daysInMonth = new Date(currentYear, today.getMonth() + 1, 0).getDate()
  const firstDayOfMonth = new Date(currentYear, today.getMonth(), 1).getDay()

  const calendarDays = []
  for (let i = 0; i < firstDayOfMonth; i++) {
    calendarDays.push(null)
  }
  for (let i = 1; i <= daysInMonth; i++) {
    calendarDays.push(i)
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setIsLoading(true)
    await new Promise((resolve) => setTimeout(resolve, 1500))
    setIsLoading(false)
    setIsSubmitted(true)
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => {
    setFormData({ ...formData, [e.target.name]: e.target.value })
  }

  if (isSubmitted) {
    return (
      <div className="relative min-h-screen flex items-center justify-center bg-background overflow-hidden">
        <GridBackground />
        <div className="relative z-10 text-center px-4">
          <div className="w-20 h-20 rounded-full bg-primary/20 flex items-center justify-center mx-auto mb-6">
            <CheckCircle2 className="w-10 h-10 text-primary" />
          </div>
          <h1 className="text-3xl font-bold text-foreground mb-4">Demo Request Received</h1>
          <p className="text-muted-foreground max-w-md mx-auto mb-8">
            Thank you for your interest in Infiner Enterprise. Our team will reach out within 24 hours to confirm your
            demo{selectedDate && selectedTime ? ` on ${currentMonth} ${selectedDate} at ${selectedTime}` : ""}.
          </p>
          <Button asChild className="bg-primary hover:bg-primary/90">
            <Link href="/">Back to Home</Link>
          </Button>
        </div>
      </div>
    )
  }

  return (
    <div className="relative min-h-screen flex bg-background overflow-hidden">
      <GridBackground />

      <div
        className="absolute top-1/4 left-1/2 -translate-x-1/2 w-[800px] h-[400px] rounded-full pointer-events-none"
        style={{
          background: "radial-gradient(ellipse, rgba(34, 94, 223, 0.06) 0%, transparent 70%)",
        }}
      />

      {/* Left side - Value props */}
      <div className="hidden lg:flex lg:w-2/5 relative z-10 flex-col justify-between p-12">
        <Link href="/" className="flex items-center">
          <Image src="/cass-logo-small.svg" alt="Infiner" width={138} height={32} priority />
        </Link>

        <div className="max-w-sm">
          <h1 className="text-3xl font-bold text-foreground mb-4">Scale your AI infrastructure with confidence</h1>
          <p className="text-muted-foreground mb-8">
            Get a personalized demo of Infiner Enterprise and discover how we can help your organization.
          </p>

          <ul className="space-y-4">
            {benefits.map((benefit, index) => (
              <li key={index} className="flex items-center gap-3 text-muted-foreground">
                <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center shrink-0">
                  <benefit.icon className="w-5 h-5 text-primary" />
                </div>
                <span>{benefit.text}</span>
              </li>
            ))}
          </ul>
        </div>

        <div className="flex items-center gap-4">
          <div className="flex -space-x-2">
            {[1, 2, 3, 4].map((i) => (
              <div
                key={i}
                className="w-8 h-8 rounded-full bg-primary/20 border-2 border-background flex items-center justify-center"
              >
                <span className="text-xs text-primary font-medium">{String.fromCharCode(64 + i)}</span>
              </div>
            ))}
          </div>
          <p className="text-sm text-muted-foreground">
            Join <span className="text-foreground font-medium">500+</span> enterprise teams
          </p>
        </div>
      </div>

      {/* Right side - Form & Calendar */}
      <div className="w-full lg:w-3/5 relative z-10 flex items-center justify-center p-6 lg:p-12">
        <div className="w-full max-w-2xl">
          {/* Mobile logo */}
          <div className="lg:hidden mb-8">
            <Link href="/" className="flex items-center justify-center">
              <Image src="/cass-logo-small.svg" alt="Infiner" width={138} height={32} priority />
            </Link>
          </div>

          <div className="bg-card/50 backdrop-blur-sm border border-white/5 rounded-xl p-8">
            <div className="text-center mb-8 lg:text-left">
              <h2 className="text-2xl font-semibold text-foreground">Request a Demo</h2>
              <p className="text-muted-foreground mt-2 text-sm">
                Fill out the form and select a time that works for you
              </p>
            </div>

            <form onSubmit={handleSubmit} className="space-y-6">
              {/* Name row */}
              <div className="grid sm:grid-cols-2 gap-4">
                <div>
                  <label htmlFor="firstName" className="block text-sm font-medium text-foreground mb-2">
                    First name
                  </label>
                  <input
                    id="firstName"
                    name="firstName"
                    type="text"
                    value={formData.firstName}
                    onChange={handleInputChange}
                    placeholder="John"
                    required
                    className="w-full px-4 py-2.5 rounded-lg bg-white/5 border border-white/10 text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-all"
                  />
                </div>
                <div>
                  <label htmlFor="lastName" className="block text-sm font-medium text-foreground mb-2">
                    Last name
                  </label>
                  <input
                    id="lastName"
                    name="lastName"
                    type="text"
                    value={formData.lastName}
                    onChange={handleInputChange}
                    placeholder="Doe"
                    required
                    className="w-full px-4 py-2.5 rounded-lg bg-white/5 border border-white/10 text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-all"
                  />
                </div>
              </div>

              {/* Email & Company */}
              <div className="grid sm:grid-cols-2 gap-4">
                <div>
                  <label htmlFor="email" className="block text-sm font-medium text-foreground mb-2">
                    Work email
                  </label>
                  <input
                    id="email"
                    name="email"
                    type="email"
                    value={formData.email}
                    onChange={handleInputChange}
                    placeholder="you@company.com"
                    required
                    className="w-full px-4 py-2.5 rounded-lg bg-white/5 border border-white/10 text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-all"
                  />
                </div>
                <div>
                  <label htmlFor="company" className="block text-sm font-medium text-foreground mb-2">
                    Company
                  </label>
                  <input
                    id="company"
                    name="company"
                    type="text"
                    value={formData.company}
                    onChange={handleInputChange}
                    placeholder="Acme Inc."
                    required
                    className="w-full px-4 py-2.5 rounded-lg bg-white/5 border border-white/10 text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-all"
                  />
                </div>
              </div>

              {/* Job Title & Team Size */}
              <div className="grid sm:grid-cols-2 gap-4">
                <div>
                  <label htmlFor="jobTitle" className="block text-sm font-medium text-foreground mb-2">
                    Job title
                  </label>
                  <input
                    id="jobTitle"
                    name="jobTitle"
                    type="text"
                    value={formData.jobTitle}
                    onChange={handleInputChange}
                    placeholder="CTO"
                    required
                    className="w-full px-4 py-2.5 rounded-lg bg-white/5 border border-white/10 text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-all"
                  />
                </div>
                <div>
                  <label htmlFor="teamSize" className="block text-sm font-medium text-foreground mb-2">
                    Team size
                  </label>
                  <select
                    id="teamSize"
                    name="teamSize"
                    value={formData.teamSize}
                    onChange={handleInputChange}
                    required
                    className="w-full px-4 py-2.5 rounded-lg bg-white/5 border border-white/10 text-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-all appearance-none"
                  >
                    <option value="" disabled>
                      Select team size
                    </option>
                    <option value="1-10">1-10 employees</option>
                    <option value="11-50">11-50 employees</option>
                    <option value="51-200">51-200 employees</option>
                    <option value="201-500">201-500 employees</option>
                    <option value="500+">500+ employees</option>
                  </select>
                </div>
              </div>

              {/* Calendar section */}
              <div className="border-t border-white/10 pt-6">
                <div className="flex items-center gap-2 mb-4">
                  <Calendar className="w-4 h-4 text-primary" />
                  <span className="text-sm font-medium text-foreground">Select a date & time</span>
                </div>

                <div className="grid sm:grid-cols-2 gap-6">
                  {/* Mini calendar */}
                  <div className="bg-white/5 rounded-lg p-4 border border-white/10">
                    <div className="text-center mb-4">
                      <span className="text-sm font-medium text-foreground">
                        {currentMonth} {currentYear}
                      </span>
                    </div>
                    <div className="grid grid-cols-7 gap-1 text-center text-xs mb-2">
                      {["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"].map((day) => (
                        <span key={day} className="text-muted-foreground py-1">
                          {day}
                        </span>
                      ))}
                    </div>
                    <div className="grid grid-cols-7 gap-1">
                      {calendarDays.map((day, index) => {
                        const isPast = day !== null && day < today.getDate()
                        const isWeekend =
                          day !== null &&
                          (new Date(currentYear, today.getMonth(), day).getDay() === 0 ||
                            new Date(currentYear, today.getMonth(), day).getDay() === 6)
                        const isDisabled = isPast || isWeekend

                        return (
                          <button
                            key={index}
                            type="button"
                            disabled={day === null || isDisabled}
                            onClick={() => day && !isDisabled && setSelectedDate(day)}
                            className={`
                              aspect-square rounded text-xs flex items-center justify-center transition-all
                              ${day === null ? "invisible" : ""}
                              ${isDisabled ? "text-muted-foreground/30 cursor-not-allowed" : "text-muted-foreground hover:bg-primary/20 hover:text-foreground"}
                              ${selectedDate === day ? "bg-primary text-primary-foreground" : ""}
                            `}
                          >
                            {day}
                          </button>
                        )
                      })}
                    </div>
                  </div>

                  {/* Time slots */}
                  <div>
                    <div className="flex items-center gap-2 mb-3">
                      <Clock className="w-4 h-4 text-muted-foreground" />
                      <span className="text-xs text-muted-foreground">Available times (EST)</span>
                    </div>
                    <div className="grid grid-cols-2 gap-2">
                      {timeSlots.map((time) => (
                        <button
                          key={time}
                          type="button"
                          onClick={() => setSelectedTime(time)}
                          className={`
                            px-3 py-2 rounded-lg text-sm transition-all border
                            ${
                              selectedTime === time
                                ? "bg-primary text-primary-foreground border-primary"
                                : "bg-white/5 border-white/10 text-muted-foreground hover:bg-white/10 hover:text-foreground"
                            }
                          `}
                        >
                          {time}
                        </button>
                      ))}
                    </div>
                    {selectedDate && selectedTime && (
                      <div className="mt-4 flex items-center gap-2 text-sm text-primary">
                        <Video className="w-4 h-4" />
                        <span>30 min video call</span>
                      </div>
                    )}
                  </div>
                </div>
              </div>

              {/* Message */}
              <div>
                <label htmlFor="message" className="block text-sm font-medium text-foreground mb-2">
                  Anything else we should know? (optional)
                </label>
                <textarea
                  id="message"
                  name="message"
                  value={formData.message}
                  onChange={handleInputChange}
                  placeholder="Tell us about your use case..."
                  rows={3}
                  className="w-full px-4 py-2.5 rounded-lg bg-white/5 border border-white/10 text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-all resize-none"
                />
              </div>

              <Button
                type="submit"
                className="w-full bg-primary hover:bg-primary/90 text-primary-foreground py-2.5"
                disabled={isLoading || !selectedDate || !selectedTime}
              >
                {isLoading ? (
                  <span className="flex items-center gap-2">
                    <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                      <circle
                        className="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        strokeWidth="4"
                        fill="none"
                      />
                      <path
                        className="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                      />
                    </svg>
                    Scheduling...
                  </span>
                ) : (
                  "Schedule Demo"
                )}
              </Button>
            </form>
          </div>

          <p className="mt-6 text-center text-sm text-muted-foreground">
            Not ready for a demo?{" "}
            <Link href="/signup" className="text-primary hover:underline font-medium">
              Start with Developer plan
            </Link>
          </p>
        </div>
      </div>
    </div>
  )
}
