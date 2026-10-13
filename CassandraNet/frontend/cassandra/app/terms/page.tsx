import Link from "next/link"
import { Navbar } from "@/components/navbar"
import { Footer } from "@/components/footer"
import { GridBackground } from "@/components/grid-background"
import { ScrollToTop } from "@/components/scroll-to-top"

const sections = [
  {
    title: "1. Acceptance of Terms",
    content: `By accessing or using Infiner's services, you agree to be bound by these Terms of Service and all applicable laws and regulations. If you do not agree with any of these terms, you are prohibited from using or accessing our services.

These terms apply to all users, including visitors, registered users, and API consumers. We reserve the right to modify these terms at any time, and your continued use of the service constitutes acceptance of those modifications.`,
  },
  {
    title: "2. Description of Service",
    content: `Infiner provides AI inference infrastructure and API services that enable developers and organizations to access and deploy machine learning models. Our services include:

• API access to various AI models
• Model hosting and deployment infrastructure
• Usage analytics and monitoring tools
• Developer documentation and support

We reserve the right to modify, suspend, or discontinue any aspect of our services at any time without prior notice.`,
  },
  {
    title: "3. Account Registration",
    content: `To access certain features of our services, you must register for an account. You agree to:

• Provide accurate, current, and complete information during registration
• Maintain and promptly update your account information
• Keep your password secure and confidential
• Accept responsibility for all activities under your account
• Notify us immediately of any unauthorized use

We reserve the right to suspend or terminate accounts that violate these terms or engage in fraudulent activity.`,
  },
  {
    title: "4. API Usage and Rate Limits",
    content: `Your use of Infiner APIs is subject to rate limits and usage quotas as specified in your plan. You agree to:

• Not exceed your allocated rate limits or attempt to circumvent them
• Not use the API for any illegal or unauthorized purpose
• Not interfere with or disrupt the integrity of our services
• Not attempt to gain unauthorized access to our systems
• Implement appropriate error handling and retry logic

Excessive usage or abuse may result in throttling, suspension, or termination of your API access.`,
  },
  {
    title: "5. Payment Terms",
    content: `For paid services, you agree to pay all fees according to your selected plan. Payment terms include:

• Fees are billed in advance on a monthly or annual basis
• All fees are non-refundable unless otherwise specified
• Usage-based charges are billed in arrears
• We may change pricing with 30 days notice
• Failure to pay may result in service suspension

You are responsible for all taxes associated with your use of our services, excluding taxes based on our net income.`,
  },
  {
    title: "6. Intellectual Property",
    content: `Infiner retains all rights, title, and interest in our services, including all intellectual property rights. You retain ownership of:

• Your application code and configurations
• Data you submit to our services
• Outputs generated through your use of the API

You grant us a limited license to use your content solely to provide and improve our services. We do not claim ownership of your inputs or outputs.`,
  },
  {
    title: "7. Prohibited Uses",
    content: `You agree not to use our services to:

• Generate illegal, harmful, or abusive content
• Harass, abuse, or harm others
• Violate any applicable laws or regulations
• Infringe on intellectual property rights
• Distribute malware or engage in phishing
• Attempt to reverse engineer our models or systems
• Resell API access without authorization
• Generate content that exploits minors

Violation of these restrictions may result in immediate termination without refund.`,
  },
  {
    title: "8. Limitation of Liability",
    content: `TO THE MAXIMUM EXTENT PERMITTED BY LAW, INFINER SHALL NOT BE LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL, CONSEQUENTIAL, OR PUNITIVE DAMAGES, INCLUDING LOSS OF PROFITS, DATA, OR GOODWILL.

Our total liability for any claims arising from these terms or your use of our services shall not exceed the amount you paid us in the twelve months preceding the claim.`,
  },
  {
    title: "9. Indemnification",
    content: `You agree to indemnify, defend, and hold harmless Infiner and its officers, directors, employees, and agents from any claims, damages, losses, liabilities, and expenses (including legal fees) arising from:

• Your use of our services
• Your violation of these terms
• Your violation of any third-party rights
• Any content you submit through our services`,
  },
  {
    title: "10. Termination",
    content: `Either party may terminate this agreement at any time. Upon termination:

• Your access to the services will be revoked
• Any outstanding fees become immediately due
• Provisions that should survive termination will remain in effect
• We may delete your data after a reasonable retention period

You may export your data before termination using our API or dashboard tools.`,
  },
  {
    title: "11. Governing Law",
    content: `These terms shall be governed by and construed in accordance with the laws of the State of Delaware, without regard to its conflict of law provisions.

Any disputes arising from these terms or your use of our services shall be resolved through binding arbitration in accordance with the rules of the American Arbitration Association.`,
  },
  {
    title: "12. Contact Information",
    content: `If you have any questions about these Terms of Service, please contact us at:

Email: legal@infiner.ai
Address: 123 AI Boulevard, Suite 400, San Francisco, CA 94105

For general support inquiries, please visit our documentation or contact support@infiner.ai.`,
  },
]

export default function TermsPage() {
  return (
    <div className="min-h-screen bg-background">
      <ScrollToTop />
      <Navbar />
      <GridBackground />

      {/* Header */}
      <div className="relative pt-32 pb-16">
        <div className="container mx-auto px-6">
          <div className="max-w-3xl mx-auto text-center">
            <h1 className="text-4xl md:text-5xl font-bold text-foreground mb-4">Terms of Service</h1>
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
                Welcome to Infiner. These Terms of Service ("Terms") govern your access to and use of Infiner's
                products, services, and APIs. Please read these terms carefully before using our services.
              </p>

              <div className="space-y-8">
                {sections.map((section, index) => (
                  <section key={index} className="border-b border-border/50 pb-8 last:border-0 last:pb-0">
                    <h2 className="text-xl font-semibold text-foreground mb-4">{section.title}</h2>
                    <div className="text-muted-foreground leading-relaxed whitespace-pre-line">{section.content}</div>
                  </section>
                ))}
              </div>

              <div className="mt-12 pt-8 border-t border-border/50">
                <p className="text-sm text-muted-foreground">
                  By using Infiner, you acknowledge that you have read, understood, and agree to be bound by these Terms
                  of Service. See also our{" "}
                  <Link href="/privacy" className="text-primary hover:underline">
                    Privacy Policy
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
