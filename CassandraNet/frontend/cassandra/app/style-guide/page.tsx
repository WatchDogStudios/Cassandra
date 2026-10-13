import { Navbar } from "@/components/navbar"
import { Footer } from "@/components/footer"
import { GridBackground } from "@/components/grid-background"
import { ScrollToTop } from "@/components/scroll-to-top"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Switch } from "@/components/ui/switch"
import { Checkbox } from "@/components/ui/checkbox"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { AlertCircle, Check, Copy, Info, Zap, ArrowRight, Github, Twitter } from "lucide-react"
import { SyntaxHighlighter } from "@/lib/syntax-highlighter"

export default function StyleGuidePage() {
  const colors = [
    { name: "Background", variable: "--background", class: "bg-background" },
    { name: "Foreground", variable: "--foreground", class: "bg-foreground" },
    { name: "Primary", variable: "--primary", class: "bg-primary" },
    { name: "Secondary", variable: "--secondary", class: "bg-secondary" },
    { name: "Muted", variable: "--muted", class: "bg-muted" },
    { name: "Accent", variable: "--accent", class: "bg-accent" },
    { name: "Card", variable: "--card", class: "bg-card" },
    { name: "Border", variable: "--border", class: "bg-border" },
    { name: "Destructive", variable: "--destructive", class: "bg-destructive" },
  ]

  const surfaceColors = [
    { name: "Surface Dark", variable: "--surface-dark", class: "bg-surface-dark" },
    { name: "Surface Elevated", variable: "--surface-elevated", class: "bg-surface-elevated" },
    { name: "Surface Testimonial", variable: "--surface-testimonial", class: "bg-surface-testimonial" },
  ]

  const chartColors = [
    { name: "Chart 1", variable: "--chart-1", class: "bg-chart-1" },
    { name: "Chart 2", variable: "--chart-2", class: "bg-chart-2" },
    { name: "Chart 3", variable: "--chart-3", class: "bg-chart-3" },
    { name: "Chart 4", variable: "--chart-4", class: "bg-chart-4" },
    { name: "Chart 5", variable: "--chart-5", class: "bg-chart-5" },
  ]

  const codeExample = `import { createClient } from '@infiner/sdk'

const client = createClient({
  apiKey: 'YOUR_API_KEY',
  model: 'gpt-4-turbo'
})

const response = await client.chat({
  messages: [
    { role: 'user', content: 'Hello!' }
  ],
  temperature: 0.7,
  maxTokens: 1024
})`

  return (
    <div className="min-h-screen bg-background">
      <ScrollToTop />
      <GridBackground />
      <Navbar />

      <main className="relative z-10 pt-24 pb-20">
        <div className="container mx-auto px-4 md:px-6">
          {/* Header */}
          <div className="max-w-3xl mx-auto text-center mb-16">
            <Badge className="mb-4">Style Guide</Badge>
            <h1 className="text-4xl md:text-5xl font-bold mb-4">Design System</h1>
            <p className="text-lg text-muted-foreground">
              A comprehensive guide to the colors, typography, components, and patterns used throughout the Infiner
              template.
            </p>
          </div>

          {/* Table of Contents */}
          <Card className="max-w-3xl mx-auto mb-16 bg-card/50 border-border/50">
            <CardHeader>
              <CardTitle className="text-lg">Quick Navigation</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-2">
                {[
                  "Colors",
                  "Typography",
                  "Buttons",
                  "Form Elements",
                  "Cards",
                  "Badges",
                  "Code Blocks",
                  "Animations",
                ].map((item) => (
                  <a
                    key={item}
                    href={`#${item.toLowerCase().replace(" ", "-")}`}
                    className="px-3 py-1.5 text-sm bg-secondary hover:bg-secondary/80 rounded-md transition-colors"
                  >
                    {item}
                  </a>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Colors Section */}
          <section id="colors" className="mb-20">
            <div className="max-w-5xl mx-auto">
              <h2 className="text-2xl font-bold mb-2">Colors</h2>
              <p className="text-muted-foreground mb-8">
                Theme-aware color tokens that automatically adapt to light and dark modes.
              </p>

              {/* Core Colors */}
              <h3 className="text-lg font-semibold mb-4">Core Colors</h3>
              <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4 mb-10">
                {colors.map((color) => (
                  <div key={color.name} className="space-y-2">
                    <div className={`h-20 rounded-lg border border-border/50 ${color.class}`} />
                    <div>
                      <p className="font-medium text-sm">{color.name}</p>
                      <p className="text-xs text-muted-foreground font-mono">{color.variable}</p>
                    </div>
                  </div>
                ))}
              </div>

              {/* Surface Colors */}
              <h3 className="text-lg font-semibold mb-4">Surface Colors</h3>
              <div className="grid grid-cols-2 md:grid-cols-3 gap-4 mb-10">
                {surfaceColors.map((color) => (
                  <div key={color.name} className="space-y-2">
                    <div className={`h-20 rounded-lg border border-border/50 ${color.class}`} />
                    <div>
                      <p className="font-medium text-sm">{color.name}</p>
                      <p className="text-xs text-muted-foreground font-mono">{color.variable}</p>
                    </div>
                  </div>
                ))}
              </div>

              {/* Chart Colors */}
              <h3 className="text-lg font-semibold mb-4">Chart Colors</h3>
              <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
                {chartColors.map((color) => (
                  <div key={color.name} className="space-y-2">
                    <div className={`h-16 rounded-lg ${color.class}`} />
                    <div>
                      <p className="font-medium text-sm">{color.name}</p>
                      <p className="text-xs text-muted-foreground font-mono">{color.variable}</p>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </section>

          {/* Typography Section */}
          <section id="typography" className="mb-20">
            <div className="max-w-5xl mx-auto">
              <h2 className="text-2xl font-bold mb-2">Typography</h2>
              <p className="text-muted-foreground mb-8">
                Work Sans is the primary font family, paired with Geist Mono for code.
              </p>

              <Card className="bg-card/50 border-border/50 mb-8">
                <CardContent className="pt-6 space-y-6">
                  <div className="space-y-4">
                    <div className="flex items-baseline justify-between border-b border-border/50 pb-4">
                      <h1 className="text-5xl font-bold">Heading 1</h1>
                      <span className="text-sm text-muted-foreground font-mono">text-5xl font-bold</span>
                    </div>
                    <div className="flex items-baseline justify-between border-b border-border/50 pb-4">
                      <h2 className="text-4xl font-bold">Heading 2</h2>
                      <span className="text-sm text-muted-foreground font-mono">text-4xl font-bold</span>
                    </div>
                    <div className="flex items-baseline justify-between border-b border-border/50 pb-4">
                      <h3 className="text-3xl font-semibold">Heading 3</h3>
                      <span className="text-sm text-muted-foreground font-mono">text-3xl font-semibold</span>
                    </div>
                    <div className="flex items-baseline justify-between border-b border-border/50 pb-4">
                      <h4 className="text-2xl font-semibold">Heading 4</h4>
                      <span className="text-sm text-muted-foreground font-mono">text-2xl font-semibold</span>
                    </div>
                    <div className="flex items-baseline justify-between border-b border-border/50 pb-4">
                      <h5 className="text-xl font-medium">Heading 5</h5>
                      <span className="text-sm text-muted-foreground font-mono">text-xl font-medium</span>
                    </div>
                    <div className="flex items-baseline justify-between border-b border-border/50 pb-4">
                      <p className="text-lg">Large Text</p>
                      <span className="text-sm text-muted-foreground font-mono">text-lg</span>
                    </div>
                    <div className="flex items-baseline justify-between border-b border-border/50 pb-4">
                      <p className="text-base">Body Text</p>
                      <span className="text-sm text-muted-foreground font-mono">text-base</span>
                    </div>
                    <div className="flex items-baseline justify-between border-b border-border/50 pb-4">
                      <p className="text-sm">Small Text</p>
                      <span className="text-sm text-muted-foreground font-mono">text-sm</span>
                    </div>
                    <div className="flex items-baseline justify-between">
                      <p className="text-sm text-muted-foreground">Muted Text</p>
                      <span className="text-sm text-muted-foreground font-mono">text-muted-foreground</span>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card className="bg-card/50 border-border/50">
                <CardHeader>
                  <CardTitle className="text-lg">Font Families</CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex items-center justify-between p-4 bg-secondary/50 rounded-lg">
                    <span className="font-sans text-lg">Work Sans - The quick brown fox</span>
                    <span className="text-sm text-muted-foreground font-mono">font-sans</span>
                  </div>
                  <div className="flex items-center justify-between p-4 bg-secondary/50 rounded-lg">
                    <span className="font-mono text-lg">Geist Mono - The quick brown fox</span>
                    <span className="text-sm text-muted-foreground font-mono">font-mono</span>
                  </div>
                </CardContent>
              </Card>
            </div>
          </section>

          {/* Buttons Section */}
          <section id="buttons" className="mb-20">
            <div className="max-w-5xl mx-auto">
              <h2 className="text-2xl font-bold mb-2">Buttons</h2>
              <p className="text-muted-foreground mb-8">
                Button variants for different use cases and visual hierarchy.
              </p>

              <Card className="bg-card/50 border-border/50">
                <CardContent className="pt-6">
                  <div className="space-y-8">
                    {/* Variants */}
                    <div>
                      <h3 className="text-sm font-medium text-muted-foreground mb-4">Variants</h3>
                      <div className="flex flex-wrap gap-4">
                        <Button>Default</Button>
                        <Button variant="secondary">Secondary</Button>
                        <Button variant="outline">Outline</Button>
                        <Button variant="ghost">Ghost</Button>
                        <Button variant="link">Link</Button>
                        <Button variant="destructive">Destructive</Button>
                      </div>
                    </div>

                    {/* Sizes */}
                    <div>
                      <h3 className="text-sm font-medium text-muted-foreground mb-4">Sizes</h3>
                      <div className="flex flex-wrap items-center gap-4">
                        <Button size="sm">Small</Button>
                        <Button size="default">Default</Button>
                        <Button size="lg">Large</Button>
                        <Button size="icon">
                          <Zap className="h-4 w-4" />
                        </Button>
                      </div>
                    </div>

                    {/* With Icons */}
                    <div>
                      <h3 className="text-sm font-medium text-muted-foreground mb-4">With Icons</h3>
                      <div className="flex flex-wrap gap-4">
                        <Button>
                          <Github className="mr-2 h-4 w-4" />
                          GitHub
                        </Button>
                        <Button variant="outline">
                          Continue
                          <ArrowRight className="ml-2 h-4 w-4" />
                        </Button>
                        <Button variant="secondary">
                          <Copy className="mr-2 h-4 w-4" />
                          Copy Code
                        </Button>
                      </div>
                    </div>

                    {/* States */}
                    <div>
                      <h3 className="text-sm font-medium text-muted-foreground mb-4">States</h3>
                      <div className="flex flex-wrap gap-4">
                        <Button disabled>Disabled</Button>
                        <Button className="pointer-events-none opacity-50">Loading...</Button>
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
          </section>

          {/* Form Elements Section */}
          <section id="form-elements" className="mb-20">
            <div className="max-w-5xl mx-auto">
              <h2 className="text-2xl font-bold mb-2">Form Elements</h2>
              <p className="text-muted-foreground mb-8">Input components for building forms and capturing user data.</p>

              <div className="grid md:grid-cols-2 gap-6">
                <Card className="bg-card/50 border-border/50">
                  <CardHeader>
                    <CardTitle className="text-lg">Text Inputs</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="space-y-2">
                      <Label htmlFor="email">Email</Label>
                      <Input id="email" type="email" placeholder="you@example.com" />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="password">Password</Label>
                      <Input id="password" type="password" placeholder="••••••••" />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="disabled">Disabled</Label>
                      <Input id="disabled" disabled placeholder="Disabled input" />
                    </div>
                  </CardContent>
                </Card>

                <Card className="bg-card/50 border-border/50">
                  <CardHeader>
                    <CardTitle className="text-lg">Textarea & Select</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="space-y-2">
                      <Label htmlFor="message">Message</Label>
                      <Textarea id="message" placeholder="Type your message..." />
                    </div>
                    <div className="space-y-2">
                      <Label>Select Model</Label>
                      <Select>
                        <SelectTrigger>
                          <SelectValue placeholder="Choose a model" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="gpt-4">GPT-4 Turbo</SelectItem>
                          <SelectItem value="claude">Claude 3 Opus</SelectItem>
                          <SelectItem value="gemini">Gemini Pro</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </CardContent>
                </Card>

                <Card className="bg-card/50 border-border/50">
                  <CardHeader>
                    <CardTitle className="text-lg">Toggles & Checks</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-6">
                    <div className="flex items-center justify-between">
                      <Label htmlFor="notifications">Enable notifications</Label>
                      <Switch id="notifications" />
                    </div>
                    <div className="flex items-center justify-between">
                      <Label htmlFor="streaming">Stream responses</Label>
                      <Switch id="streaming" defaultChecked />
                    </div>
                    <div className="space-y-3">
                      <div className="flex items-center space-x-2">
                        <Checkbox id="terms" />
                        <Label htmlFor="terms" className="text-sm">
                          Accept terms and conditions
                        </Label>
                      </div>
                      <div className="flex items-center space-x-2">
                        <Checkbox id="marketing" defaultChecked />
                        <Label htmlFor="marketing" className="text-sm">
                          Receive marketing emails
                        </Label>
                      </div>
                    </div>
                  </CardContent>
                </Card>

                <Card className="bg-card/50 border-border/50">
                  <CardHeader>
                    <CardTitle className="text-lg">Tabs</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <Tabs defaultValue="overview" className="w-full">
                      <TabsList className="w-full">
                        <TabsTrigger value="overview" className="flex-1">
                          Overview
                        </TabsTrigger>
                        <TabsTrigger value="analytics" className="flex-1">
                          Analytics
                        </TabsTrigger>
                        <TabsTrigger value="settings" className="flex-1">
                          Settings
                        </TabsTrigger>
                      </TabsList>
                      <TabsContent value="overview" className="mt-4 p-4 bg-secondary/50 rounded-lg">
                        <p className="text-sm text-muted-foreground">Overview content goes here.</p>
                      </TabsContent>
                      <TabsContent value="analytics" className="mt-4 p-4 bg-secondary/50 rounded-lg">
                        <p className="text-sm text-muted-foreground">Analytics content goes here.</p>
                      </TabsContent>
                      <TabsContent value="settings" className="mt-4 p-4 bg-secondary/50 rounded-lg">
                        <p className="text-sm text-muted-foreground">Settings content goes here.</p>
                      </TabsContent>
                    </Tabs>
                  </CardContent>
                </Card>
              </div>
            </div>
          </section>

          {/* Cards Section */}
          <section id="cards" className="mb-20">
            <div className="max-w-5xl mx-auto">
              <h2 className="text-2xl font-bold mb-2">Cards</h2>
              <p className="text-muted-foreground mb-8">Card components for grouping related content.</p>

              <div className="grid md:grid-cols-3 gap-6">
                <Card className="bg-card/50 border-border/50">
                  <CardHeader>
                    <CardTitle>Basic Card</CardTitle>
                    <CardDescription>A simple card with header and content.</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <p className="text-sm text-muted-foreground">
                      Cards are versatile containers that can hold any type of content.
                    </p>
                  </CardContent>
                </Card>

                <Card className="bg-card/50 border-border/50 hover:border-primary/50 transition-colors">
                  <CardHeader>
                    <div className="w-10 h-10 bg-primary/10 rounded-lg flex items-center justify-center mb-2">
                      <Zap className="w-5 h-5 text-primary" />
                    </div>
                    <CardTitle>Feature Card</CardTitle>
                    <CardDescription>With icon and hover effect.</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <p className="text-sm text-muted-foreground">Great for showcasing features or services.</p>
                  </CardContent>
                </Card>

                <Card className="bg-gradient-to-br from-primary/10 to-primary/5 border-primary/20">
                  <CardHeader>
                    <CardTitle>Gradient Card</CardTitle>
                    <CardDescription>With gradient background.</CardDescription>
                  </CardHeader>
                  <CardContent>
                    <p className="text-sm text-muted-foreground">Use gradients to highlight important content.</p>
                    <Button className="mt-4 w-full">Get Started</Button>
                  </CardContent>
                </Card>
              </div>
            </div>
          </section>

          {/* Badges Section */}
          <section id="badges" className="mb-20">
            <div className="max-w-5xl mx-auto">
              <h2 className="text-2xl font-bold mb-2">Badges</h2>
              <p className="text-muted-foreground mb-8">Small labels for status indicators and categories.</p>

              <Card className="bg-card/50 border-border/50">
                <CardContent className="pt-6">
                  <div className="space-y-6">
                    <div>
                      <h3 className="text-sm font-medium text-muted-foreground mb-4">Variants</h3>
                      <div className="flex flex-wrap gap-3">
                        <Badge>Default</Badge>
                        <Badge variant="secondary">Secondary</Badge>
                        <Badge variant="outline">Outline</Badge>
                        <Badge variant="destructive">Destructive</Badge>
                      </div>
                    </div>

                    <div>
                      <h3 className="text-sm font-medium text-muted-foreground mb-4">With Icons</h3>
                      <div className="flex flex-wrap gap-3">
                        <Badge className="gap-1">
                          <Check className="w-3 h-3" />
                          Success
                        </Badge>
                        <Badge variant="secondary" className="gap-1">
                          <Info className="w-3 h-3" />
                          Info
                        </Badge>
                        <Badge variant="destructive" className="gap-1">
                          <AlertCircle className="w-3 h-3" />
                          Error
                        </Badge>
                      </div>
                    </div>

                    <div>
                      <h3 className="text-sm font-medium text-muted-foreground mb-4">Use Cases</h3>
                      <div className="flex flex-wrap gap-3">
                        <Badge className="bg-emerald-500/10 text-emerald-500 hover:bg-emerald-500/20">Online</Badge>
                        <Badge className="bg-amber-500/10 text-amber-500 hover:bg-amber-500/20">In Progress</Badge>
                        <Badge className="bg-blue-500/10 text-blue-500 hover:bg-blue-500/20">New</Badge>
                        <Badge className="bg-purple-500/10 text-purple-500 hover:bg-purple-500/20">Beta</Badge>
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
          </section>

          {/* Code Blocks Section */}
          <section id="code-blocks" className="mb-20">
            <div className="max-w-5xl mx-auto">
              <h2 className="text-2xl font-bold mb-2">Code Blocks</h2>
              <p className="text-muted-foreground mb-8">Syntax-highlighted code blocks with brand-specific colors.</p>

              <Card className="bg-card/50 border-border/50">
                <CardHeader>
                  <CardTitle className="text-lg">JavaScript / TypeScript</CardTitle>
                  <CardDescription>Brand-aware syntax highlighting</CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="bg-[#0d1117] rounded-lg border border-border/50 overflow-hidden">
                    <div className="flex items-center justify-between px-4 py-2 border-b border-border/50 bg-secondary/20">
                      <span className="text-xs text-muted-foreground font-mono">example.ts</span>
                      <Button variant="ghost" size="sm" className="h-6 px-2">
                        <Copy className="w-3 h-3 mr-1" />
                        <span className="text-xs">Copy</span>
                      </Button>
                    </div>
                    <div className="p-4">
                      <SyntaxHighlighter code={codeExample} language="typescript" />
                    </div>
                  </div>
                </CardContent>
              </Card>
            </div>
          </section>

          {/* Animations Section */}
          <section id="animations" className="mb-20">
            <div className="max-w-5xl mx-auto">
              <h2 className="text-2xl font-bold mb-2">Animations</h2>
              <p className="text-muted-foreground mb-8">CSS animations available throughout the template.</p>

              <div className="grid md:grid-cols-2 gap-6">
                <Card className="bg-card/50 border-border/50">
                  <CardHeader>
                    <CardTitle className="text-lg">Available Classes</CardTitle>
                  </CardHeader>
                  <CardContent>
                    <div className="space-y-3">
                      {[
                        { class: "animate-navbar-drop", desc: "Navbar drop-in from top" },
                        { class: "animate-slide-in-right", desc: "Slide in from right" },
                        { class: "animate-slide-in-left", desc: "Slide in from left" },
                        { class: "animate-slide-up-delayed", desc: "Slide up with delay" },
                        { class: "animate-fade-in-delayed", desc: "Fade in with delay" },
                        { class: "animate-marquee", desc: "Infinite horizontal scroll" },
                        { class: "animate-scroll-up", desc: "Vertical scroll up" },
                        { class: "animate-scroll-down", desc: "Vertical scroll down" },
                        { class: "animate-meteor-effect", desc: "Meteor falling effect" },
                      ].map((item) => (
                        <div key={item.class} className="flex items-center justify-between p-2 bg-secondary/50 rounded">
                          <code className="text-sm font-mono text-primary">{item.class}</code>
                          <span className="text-xs text-muted-foreground">{item.desc}</span>
                        </div>
                      ))}
                    </div>
                  </CardContent>
                </Card>

                <Card className="bg-card/50 border-border/50">
                  <CardHeader>
                    <CardTitle className="text-lg">Hover Effects</CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="p-4 bg-secondary/50 rounded-lg">
                      <p className="text-sm text-muted-foreground mb-3">Nav Link Hover</p>
                      <a href="#" className="nav-link-hover text-foreground font-medium">
                        Hover over me
                      </a>
                    </div>
                    <div className="p-4 bg-secondary/50 rounded-lg">
                      <p className="text-sm text-muted-foreground mb-3">Button Hover States</p>
                      <div className="flex gap-2">
                        <Button size="sm">Scale</Button>
                        <Button size="sm" variant="outline">
                          Border
                        </Button>
                        <Button size="sm" variant="ghost">
                          Opacity
                        </Button>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              </div>
            </div>
          </section>

          {/* Icons Section */}
          <section id="icons" className="mb-20">
            <div className="max-w-5xl mx-auto">
              <h2 className="text-2xl font-bold mb-2">Icons</h2>
              <p className="text-muted-foreground mb-8">Lucide React icons are used throughout the template.</p>

              <Card className="bg-card/50 border-border/50">
                <CardContent className="pt-6">
                  <div className="grid grid-cols-4 md:grid-cols-8 gap-4">
                    {[
                      { icon: Zap, name: "Zap" },
                      { icon: Check, name: "Check" },
                      { icon: Copy, name: "Copy" },
                      { icon: Info, name: "Info" },
                      { icon: AlertCircle, name: "AlertCircle" },
                      { icon: ArrowRight, name: "ArrowRight" },
                      { icon: Github, name: "Github" },
                      { icon: Twitter, name: "Twitter" },
                    ].map(({ icon: Icon, name }) => (
                      <div key={name} className="flex flex-col items-center gap-2 p-3 bg-secondary/50 rounded-lg">
                        <Icon className="w-5 h-5" />
                        <span className="text-xs text-muted-foreground">{name}</span>
                      </div>
                    ))}
                  </div>
                  <p className="text-sm text-muted-foreground mt-4">
                    Import from <code className="text-primary">lucide-react</code> -
                    <a
                      href="https://lucide.dev/icons"
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-primary hover:underline ml-1"
                    >
                      View all icons
                    </a>
                  </p>
                </CardContent>
              </Card>
            </div>
          </section>

          {/* Components List */}
          <section className="mb-20">
            <div className="max-w-5xl mx-auto">
              <h2 className="text-2xl font-bold mb-2">Available Components</h2>
              <p className="text-muted-foreground mb-8">All shadcn/ui components available in this template.</p>

              <Card className="bg-card/50 border-border/50">
                <CardContent className="pt-6">
                  <div className="flex flex-wrap gap-2">
                    {[
                      "Accordion",
                      "Alert",
                      "AlertDialog",
                      "AspectRatio",
                      "Avatar",
                      "Badge",
                      "Breadcrumb",
                      "Button",
                      "Calendar",
                      "Card",
                      "Carousel",
                      "Chart",
                      "Checkbox",
                      "Collapsible",
                      "Command",
                      "ContextMenu",
                      "Dialog",
                      "Drawer",
                      "DropdownMenu",
                      "Form",
                      "HoverCard",
                      "Input",
                      "Label",
                      "Menubar",
                      "NavigationMenu",
                      "Pagination",
                      "Popover",
                      "Progress",
                      "RadioGroup",
                      "ScrollArea",
                      "Select",
                      "Separator",
                      "Sheet",
                      "Skeleton",
                      "Slider",
                      "Sonner",
                      "Switch",
                      "Table",
                      "Tabs",
                      "Textarea",
                      "Toast",
                      "Toggle",
                      "Tooltip",
                    ].map((component) => (
                      <Badge key={component} variant="secondary" className="font-mono text-xs">
                        {component}
                      </Badge>
                    ))}
                  </div>
                </CardContent>
              </Card>
            </div>
          </section>
        </div>
      </main>

      <Footer />
    </div>
  )
}
