import Link from "next/link"
import { Navbar } from "@/components/navbar"
import { Footer } from "@/components/footer"
import { GridBackground } from "@/components/grid-background"
import { ScrollToTop } from "@/components/scroll-to-top"

const sections = [
  {
    title: "1. Information We Collect",
    content: `We collect information you provide directly to us, including:

**Account Information:** When you create an account, we collect your name, email address, company name, and password.

**Payment Information:** When you make a purchase, our payment processor collects your payment card details. We do not store full card numbers.

**Usage Data:** We automatically collect information about your use of our services, including API calls, timestamps, model selections, and performance metrics.

**Communications:** When you contact us, we collect information in your messages, including support requests and feedback.`,
  },
  {
    title: "2. How We Use Your Information",
    content: `We use the information we collect to:

• Provide, maintain, and improve our services
• Process transactions and send related information
• Send technical notices, updates, and security alerts
• Respond to your comments, questions, and support requests
• Monitor and analyze trends, usage, and activities
• Detect, investigate, and prevent fraudulent transactions and abuse
• Personalize and improve your experience
• Comply with legal obligations`,
  },
  {
    title: "3. Data Processing and AI",
    content: `**Input Data:** When you use our API, your input prompts are processed by our systems to generate responses. We do not use your inputs to train our models without explicit consent.

**Output Data:** AI-generated outputs are returned to you and may be temporarily cached for performance optimization.

**Retention:** API inputs and outputs are retained for up to 30 days for abuse prevention and debugging purposes, unless you opt out or request earlier deletion.

**Model Improvement:** We may use aggregated, anonymized usage patterns to improve our services, but this never includes identifiable content from your prompts.`,
  },
  {
    title: "4. Information Sharing",
    content: `We do not sell your personal information. We may share information in the following circumstances:

**Service Providers:** With vendors who perform services on our behalf, such as payment processing, hosting, and analytics.

**Legal Requirements:** When required by law, subpoena, or legal process.

**Business Transfers:** In connection with a merger, acquisition, or sale of assets.

**Protection:** To protect the rights, property, and safety of Infiner, our users, or others.

**With Consent:** With your explicit consent or at your direction.`,
  },
  {
    title: "5. Data Security",
    content: `We implement industry-standard security measures to protect your information:

• Encryption in transit (TLS 1.3) and at rest (AES-256)
• Regular security audits and penetration testing
• SOC 2 Type II certification
• Access controls and authentication requirements
• Monitoring and logging of system access
• Incident response procedures

While we strive to protect your information, no method of transmission over the Internet is 100% secure. We cannot guarantee absolute security.`,
  },
  {
    title: "6. Data Retention",
    content: `We retain your information for as long as your account is active or as needed to provide services. Retention periods vary by data type:

• **Account Data:** Retained until account deletion, plus 30 days
• **API Logs:** Retained for 30 days by default
• **Payment Records:** Retained for 7 years for compliance
• **Support Communications:** Retained for 3 years

You may request deletion of your data at any time, subject to legal retention requirements.`,
  },
  {
    title: "7. Your Rights and Choices",
    content: `Depending on your location, you may have the following rights:

**Access:** Request a copy of the personal information we hold about you.

**Correction:** Request correction of inaccurate personal information.

**Deletion:** Request deletion of your personal information.

**Portability:** Request a portable copy of your data.

**Objection:** Object to certain processing of your information.

**Restriction:** Request restriction of processing in certain circumstances.

To exercise these rights, contact us at privacy@infiner.ai.`,
  },
  {
    title: "8. International Data Transfers",
    content: `We are based in the United States and process information on servers located in the US and other countries. If you are located outside the US, your information will be transferred to and processed in the US.

We use appropriate safeguards for international transfers, including:

• Standard Contractual Clauses approved by the European Commission
• Data Processing Agreements with service providers
• Certification under applicable privacy frameworks`,
  },
  {
    title: "9. Cookies and Tracking",
    content: `We use cookies and similar technologies to:

• Keep you logged in and remember your preferences
• Analyze usage patterns and improve our services
• Provide personalized content and features

**Essential Cookies:** Required for basic functionality.

**Analytics Cookies:** Help us understand how you use our services.

**Preference Cookies:** Remember your settings and choices.

You can control cookies through your browser settings. Disabling certain cookies may affect functionality.`,
  },
  {
    title: "10. Children's Privacy",
    content: `Our services are not directed to children under 13 (or 16 in certain jurisdictions). We do not knowingly collect personal information from children.

If you believe we have collected information from a child, please contact us immediately at privacy@infiner.ai, and we will take steps to delete such information.`,
  },
  {
    title: "11. Changes to This Policy",
    content: `We may update this Privacy Policy from time to time. We will notify you of material changes by:

• Posting the updated policy on our website
• Sending an email to your registered address
• Displaying a prominent notice in our dashboard

Your continued use of our services after changes become effective constitutes acceptance of the revised policy.`,
  },
  {
    title: "12. Contact Us",
    content: `If you have questions about this Privacy Policy or our data practices, please contact us:

**Privacy Inquiries:** privacy@infiner.ai

**Data Protection Officer:** dpo@infiner.ai

**Mailing Address:**
Infiner, Inc.
Attn: Privacy Team
123 AI Boulevard, Suite 400
San Francisco, CA 94105

For EU residents, you also have the right to lodge a complaint with your local data protection authority.`,
  },
]

export default function PrivacyPage() {
  return (
    <div className="min-h-screen bg-background">
      <ScrollToTop />
      <Navbar />
      <GridBackground />

      {/* Header */}
      <div className="relative pt-32 pb-16">
        <div className="container mx-auto px-6">
          <div className="max-w-3xl mx-auto text-center">
            <h1 className="text-4xl md:text-5xl font-bold text-foreground mb-4">Privacy Policy</h1>
            <p className="text-muted-foreground">Last updated: December 19, 2025</p>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="relative pb-24">
        <div className="container mx-auto px-6">
          <div className="max-w-3xl mx-auto">
            <div className="bg-card/50 backdrop-blur-sm border border-border/50 rounded-xl p-8 md:p-12">
              <p className="text-muted-foreground mb-8 leading-relaxed">
                At Infiner, we take your privacy seriously. This Privacy Policy explains how we collect, use, disclose,
                and safeguard your information when you use our services. Please read this policy carefully to
                understand our practices regarding your personal data.
              </p>

              <div className="space-y-8">
                {sections.map((section, index) => (
                  <section key={index} className="border-b border-border/50 pb-8 last:border-0 last:pb-0">
                    <h2 className="text-xl font-semibold text-foreground mb-4">{section.title}</h2>
                    <div className="text-muted-foreground leading-relaxed whitespace-pre-line prose-strong:text-foreground prose-strong:font-medium">
                      {section.content.split("**").map((part, i) =>
                        i % 2 === 0 ? (
                          part
                        ) : (
                          <strong key={i} className="text-foreground font-medium">
                            {part}
                          </strong>
                        ),
                      )}
                    </div>
                  </section>
                ))}
              </div>

              <div className="mt-12 pt-8 border-t border-border/50">
                <p className="text-sm text-muted-foreground">
                  By using Infiner, you acknowledge that you have read and understood this Privacy Policy. See also our{" "}
                  <Link href="/terms" className="text-primary hover:underline">
                    Terms of Service
                  </Link>
                  .
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>

      <Footer />
    </div>
  )
}
