# Solana Intents (MVP)

An intent-centric execution layer for Solana. This repo currently includes a minimal Rust solver crate and CI to build/test it. More packages (frontend, services, on-chain) will be added incrementally.

## Repo layout
- `packages/solver`: Rust solver (binary) — builds and runs
- `packages/frontend`: placeholder for Next.js app
- `packages/services`: placeholder for FastAPI/ML service
- `.github/workflows/ci.yml`: minimal CI for solver build/test

## Quickstart (local)
1. Open a terminal in the repo root.
2. Build the solver:
   ```bash
   cd solana-intents/packages/solver
   cargo build
   ```
3. Run the solver:
   ```bash
   cargo run
   ```
4. Run tests:
   ```bash
   cargo test
   ```

You should see a startup message and a dummy priority fee printed. Tests validate the basic async flow.

## Next steps
- Frontend: wallet connect + basic intent form (Jupiter swap)
- Solver: add HTTP API (Axum) and Solana RPC/Jito fee polling
- Services: data collection API and simple heuristic endpoint

Contributions: open an issue first, get assigned and create a PR against `main` with branch name `feature/your-feature-name`. Keep changes scoped and add tests where feasible.
