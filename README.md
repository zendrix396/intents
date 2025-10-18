# Solana Intents

An intent-centric execution layer for Solana. Less instructing, more achieving.

## What's This All About?

Instead of telling the blockchain *how* to do something, users just state their goal (their "intent"). Our system then figures out the best way to make it happen.

> **User:** "I want to swap 1 SOL for the most USDC I can get."
>
> **System:** *Finds the best route across multiple DEXs, builds the optimal transaction, and executes it.*

Want the full breakdown of the vision, who this is for, and how it will all work?
**Check out the vision doc: [`docs/About.md`](docs/About.md)**

---

## Getting Started

You'll need the latest stable Rust and Node.js.

1.  **Clone the repo:**
    ```bash
    git clone https://github.com/zendrix396/solana-intents.git
    cd solana-intents
    ```

2.  **Build the workspace:**
    This compiles all the Rust packages (`solver-core` and `solver-service`).
    ```bash
    cargo build --workspace
    ```

3.  **Run the tests:**
    Make sure everything is working before you start.
    ```bash
    cargo test --workspace
    ```

4.  **Run the solver service:**
    This starts the backend API server.
    ```bash
    cargo run -p solver-service
    ```
    You'll see the server start up on `http://0.0.0.0:3000`.

---

## Repo Layout

The project is a monorepo containing a few key packages:

-   `packages/solver-core`: The brain. A Rust library with all the core intent-solving logic.
-   `packages/solver-service`: The server. A lightweight Rust binary that runs `solver-core` and exposes it via an Axum web server.
-   `packages/frontend`: The face. A Next.js app for the UI. (WIP)
-   `docs/`: All project documentation.

For a deeper dive into the "why" behind this structure, see [`docs/Architecture.md`](docs/Architecture.md).

---

## Want to Contribute?

Cool. We'd love the help. The basic flow is to find an issue, get assigned, and create a PR.

All the specific details on our branching strategy, how to format your PRs, and how to keep your fork in sync are in our contribution guide.

**Please read it here: [`CONTRIBUTING.md`](CONTRIBUTING.md)**

---

## Documentation

All our documentation lives in the `/docs` folder and will eventually be available on our GitHub Pages site.

-   **[About the Project](docs/About.md):** What this is and where it's going.
-   **[Architecture Deep Dive](docs/Architecture.md):** How the pieces fit together.
-   **[Contribution Guide](CONTRIBUTING.md):** How to help out.