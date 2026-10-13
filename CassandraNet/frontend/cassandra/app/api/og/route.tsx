// import { ImageResponse } from "@vercel/og"
import type { NextRequest } from "next/server"

export const runtime = "edge"

export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url)
  const title = searchParams.get("title") || "Infiner"
  const description = searchParams.get("description") || "Infinitely Scaleable Inference"

  /*
  return new ImageResponse(
    (
      <div
        style={{
          height: "100%",
          width: "100%",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: "#030712",
          backgroundImage: "radial-gradient(circle at 50% 50%, #1e3a8a 0%, transparent 50%)",
        }}
      >
        <div
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            marginBottom: 40,
          }}
        >
          <svg width="64" height="64" viewBox="0 0 32 32" fill="none">
            <rect width="32" height="32" rx="8" fill="url(#gradient)" />
            <path
              d="M8 16C8 13.5 10 11.5 12.5 11.5C15 11.5 16 13 16 13C16 13 17 11.5 19.5 11.5C22 11.5 24 13.5 24 16C24 18.5 22 20.5 19.5 20.5C17 20.5 16 19 16 19C16 19 15 20.5 12.5 20.5C10 20.5 8 18.5 8 16Z"
              stroke="white"
              strokeWidth="1.5"
              fill="none"
            />
            <defs>
              <linearGradient id="gradient" x1="0" y1="0" x2="32" y2="32">
                <stop stopColor="#3b82f6" />
                <stop offset="1" stopColor="#1d4ed8" />
              </linearGradient>
            </defs>
          </svg>
          <span
            style={{
              marginLeft: 16,
              fontSize: 48,
              fontWeight: 700,
              color: "white",
              letterSpacing: "-0.02em",
            }}
          >
            INFINER
          </span>
        </div>
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            maxWidth: 900,
            textAlign: "center",
          }}
        >
          <h1
            style={{
              fontSize: 56,
              fontWeight: 700,
              color: "white",
              lineHeight: 1.2,
              marginBottom: 16,
            }}
          >
            {title}
          </h1>
          <p
            style={{
              fontSize: 28,
              color: "#9ca3af",
              lineHeight: 1.4,
            }}
          >
            {description}
          </p>
        </div>
      </div>
    ),
    {
      width: 1200,
      height: 630,
    }
  )
  */

  // Placeholder response for v0 preview
  return new Response(
    JSON.stringify({
      message: "OG Image generation ready - uncomment ImageResponse when deployed to Vercel",
      title,
      description,
    }),
    {
      headers: { "Content-Type": "application/json" },
    },
  )
}
