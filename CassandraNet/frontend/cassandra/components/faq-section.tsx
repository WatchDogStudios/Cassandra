"use client"

import { useEffect, useRef, useState } from "react"
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/components/ui/accordion"

const faqs = [
  {
    question: "How does Infiner's pricing work?",
    answer:
      "We offer simple pay-as-you-go pricing based on token usage. You only pay for what you use with no minimum commitments. Enterprise customers can access custom pricing with volume discounts and dedicated support.",
  },
  {
    question: "What models are available on the platform?",
    answer:
      "Infiner provides access to 50+ state-of-the-art models including GPT-4, Claude 3, Llama 3, Mistral, and more. We continuously add new models as they become available, ensuring you always have access to the latest capabilities.",
  },
  {
    question: "How fast is the inference?",
    answer:
      "Our infrastructure delivers sub-100ms latency for most models with edge deployments across 200+ global locations. We optimize for both throughput and latency, with some models achieving 10,000+ tokens per second.",
  },
  {
    question: "Can I switch between models without changing my code?",
    answer:
      "Yes, Infiner's unified API allows you to seamlessly switch between any supported model with a single parameter change. No code refactoring required - just update the model name in your request.",
  },
  {
    question: "What security certifications do you have?",
    answer:
      "We are SOC 2 Type II certified and GDPR compliant. Enterprise customers can access additional security features including VPC deployment, dedicated instances, and custom data retention policies.",
  },
  {
    question: "Do you offer custom model fine-tuning?",
    answer:
      "Yes, Enterprise customers can fine-tune select models on their proprietary data. Our team provides guidance on dataset preparation, training parameters, and deployment of fine-tuned models.",
  },
]

export function FaqSection() {
  const [isVisible, setIsVisible] = useState(false)
  const sectionRef = useRef<HTMLElement>(null)

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
      {/* Subtle horizontal blur ray */}
      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[100px] rounded-full opacity-[0.08] blur-[80px] pointer-events-none bg-primary" />

      <div className="w-full flex justify-center px-4 md:px-6">
        <div className="max-w-3xl w-full">
          {/* Header */}
          <div className="text-center mb-12 md:mb-16">
            <h2
              className={`text-3xl md:text-4xl lg:text-5xl font-bold text-foreground mb-4 text-balance ${
                isVisible ? "animate-slide-up-section" : "opacity-0"
              }`}
            >
              Frequently asked questions
            </h2>
            <p
              className={`text-muted-foreground text-lg max-w-2xl mx-auto ${
                isVisible ? "animate-slide-up-section-delayed" : "opacity-0"
              }`}
            >
              Everything you need to know about Infiner and our inference platform.
            </p>
          </div>

          {/* FAQ Accordion */}
          <div
            className={`${isVisible ? "animate-slide-up-section-delayed" : "opacity-0"}`}
            style={{ animationDelay: "0.3s" }}
          >
            <Accordion type="single" collapsible className="w-full">
              {faqs.map((faq, index) => (
                <AccordionItem key={index} value={`item-${index}`} className="border-white/10 group">
                  <AccordionTrigger className="text-left text-base md:text-lg font-medium text-foreground hover:text-primary hover:no-underline py-5 transition-colors">
                    {faq.question}
                  </AccordionTrigger>
                  <AccordionContent className="text-muted-foreground text-base leading-relaxed pb-5">
                    {faq.answer}
                  </AccordionContent>
                </AccordionItem>
              ))}
            </Accordion>
          </div>
        </div>
      </div>
    </section>
  )
}
