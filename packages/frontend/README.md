# Solana Intents — Frontend

Next.js 15 web UI for the Solana intent-centric execution layer. Users connect their wallet, express a swap intent, and the solver finds the best route across DEXs.

## Tech Stack

- **Next.js 15** (App Router, React Server Components)
- **React 19** with TypeScript
- **Tailwind CSS v4** with `@tailwindcss/typography`
- **Solana Wallet Adapter** (Phantom, Solflare)
- **MDX** documentation via `next-mdx-remote/rsc`

## Getting Started

```bash
# Install dependencies
npm install

# Copy environment config
cp .env.example .env.local

# Start dev server (http://localhost:3001)
npm run dev
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `NEXT_PUBLIC_SOLVER_API_URL` | `http://localhost:3000` | Solver service API URL |

## Scripts

| Command | Description |
|---------|-------------|
| `npm run dev` | Start development server |
| `npm run build` | Production build |
| `npm run start` | Start production server |
| `npm run lint` | Run ESLint |

## Project Structure

```
src/
├── app/
│   ├── page.tsx            # Home page (swap form / hero)
│   ├── layout.tsx          # Root layout with wallet providers
│   ├── not-found.tsx       # Custom 404 page
│   ├── globals.css         # Tailwind + prose overrides
│   ├── lib/
│   │   ├── api.ts          # Solver API client
│   │   └── providers.tsx   # Solana wallet providers
│   └── docs/               # MDX documentation site
│       ├── page.tsx        # Redirects to /docs/getting-started
│       ├── layout.tsx      # Docs layout with sidebar
│       ├── [...slug]/      # Dynamic doc pages
│       ├── components/     # DocsSidebar, MdxContent
│       └── lib/docs.ts     # MDX file reader
├── components/
│   ├── Navbar.tsx          # Top navigation with wallet button
│   ├── Footer.tsx          # Site-wide footer
│   ├── IntentForm.tsx      # Token swap form
│   └── NetworkStatus.tsx   # RPC health + fee display
content/docs/               # MDX documentation content
```

## Features

- **Token Swaps** — Select tokens, get quotes, execute swaps via Jupiter aggregation
- **Slippage Control** — Configurable slippage tolerance (0.1% – 2%)
- **Network Status** — Live RPC health and priority fee monitoring
- **Documentation** — Integrated MDX docs site with sidebar navigation
- **Mobile Responsive** — Touch-friendly sidebar drawer and responsive layout
