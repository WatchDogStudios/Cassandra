# CassandraNet Frontend

This workspace hosts the React applications that surface CassandraNet to studio operators and prospects. It currently contains two experiences:

- **Marketing** (`frontend/marketing`) – A public-facing Next.js site with a product overview, feature pillars, and contact CTA.
- **Console** (`frontend/console`) – The operator dashboard built with Vite + React Query that talks to the Rust gateway.

## Prerequisites

- Node.js 18 or newer
- npm (bundled with Node 18+)

Each app keeps its own `package.json`; installs are isolated by directory.

## Quick start

### Marketing site (Next.js)
```bash
cd frontend/marketing
npm install
npm run dev
```
Visit <http://localhost:3000> to see the landing page. Static assets are powered by Tailwind CSS and the App Router.

### Operator console (Vite)
```bash
cd frontend/console
npm install
npm run dev
```
Vite serves the console at <http://localhost:5173>. Requests to `/api/*` are expected to proxy to the gateway listening at `http://127.0.0.1:8080`.

## Production builds

| App        | Command                 | Output                    |
|------------|-------------------------|---------------------------|
| Marketing  | `npm run build`         | `.next/` (server output)  |
| Console    | `npm run build`         | `dist/` (static bundle)   |

For the marketing site, run `npm run start` afterwards to launch the production server locally. The console can be previewed with `npm run preview`.

## Environment alignment

- Both apps assume the CassandraNet gateway is available at `http://127.0.0.1:8080`. Adjust proxy rules or environment variables if the backend runs elsewhere.
- Tailwind CSS drives the marketing site styling; the console relies on handcrafted CSS variables + Google Fonts to stay lightweight.

## Next steps

- Hook marketing CTAs into the eventual CRM / waitlist tooling.
- Expand the console with deeper agent insights, charts, and multi-tenant context.
