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

## Contributions: 
- open an issue first (select relevant tags), or select from the one already available and get assigned before sending the PR 
- create a PR against `main` with branch name `feature/your-feature-name`. Keep changes scoped and add tests where feasible.
- in PR, in 3 bullet points explain the changes, write `fixes #<issue-number>` on the top of PR message and send the merge request.
- always keep your local fork up to date with the original one before sending PR.
**Commands**:
```bash
 git remote add upstream https://github.com/zendrix396/intents
 git remote -v
 git fetch upstream
 git merge upstream/main
 git push -u origin feature/<your-feature>
```