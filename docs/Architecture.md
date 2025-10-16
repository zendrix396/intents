# Architecture Overview

This document outlines the architecture of the Solana Intent Execution Layer. The primary goal of this architecture is to create a system that is modular, testable, scalable, and easy for new contributors to understand.

Our design philosophy is centered around a clear **separation of concerns**, distinguishing between the core business logic, the runtime environment, on-chain contracts, and the user-facing interfaces.

## High-Level Diagram

The system is composed of several key packages that interact to translate a user's *intent* into an executed Solana transaction.

```mermaid
graph TD
    A[Frontend (Next.js)]

    subgraph Off-Chain Infrastructure
        B[Solver Service (Axum API)]
        C[Solver Core (Rust Library)]
    end

    subgraph Solana Network
        D[RPC Node]
        E[On-Chain Programs (e.g., Jupiter, Raydium)]
    end

    A -- "1. User submits intent (JSON)" --> B
    B -- "2. Validates & passes intent" --> C
    C -- "3. Queries state & prices" --> D
    C -- "4. Simulates execution paths" --> D
    C -- "5. Returns optimal solution (Transaction)" --> B
    B -- "6. Signs & sends transaction" --> D
    D -- "7. Relays to network" --> E

    style A fill:#cde4ff
    style B fill:#d2ffd2
    style C fill:#f2d2ff
    style E fill:#ffcdd2
```

## Component Breakdown

The project is structured as a monorepo containing several distinct packages, each with a specific responsibility.

### `packages/solver-core`
This is the **brain** of the entire system.

-   **Type:** Rust library crate (`lib.rs`).
-   **Responsibilities:**
    -   Defining core data structures: `Intent`, `Solution`, `ExecutionPath`.
    -   Containing all business logic for resolving intents. This includes algorithms for finding the best swap routes, calculating arbitrage opportunities, and simulating transaction outcomes.
    -   Interacting with the Solana RPC to fetch on-chain data (e.g., account states, oracle prices, liquidity pool reserves).
    -   Constructing the final, optimized Solana transaction(s) required to fulfill the intent.
-   **Key Principle:** This crate knows *what* to do, but not *how* it is run. It has no concept of a web server or a command line. This makes it highly portable and easy to unit-test.

### `packages/solver-service`
This is the **runtime engine** that exposes the `solver-core`'s logic to the outside world.

-   **Type:** Rust binary crate (`main.rs`).
-   **Dependencies:** It depends directly on `solver-core`.
-   **Responsibilities:**
    -   Setting up and running a web server (using Axum).
    -   Defining API endpoints (e.g., `GET /health`, `POST /solve`).
    -   Handling HTTP requests and responses: deserializing incoming intents from JSON, calling the appropriate functions in `solver-core`, and serializing the solution back into a JSON response.
    -   Managing its runtime environment, including configuration and logging.

### `packages/frontend` (Planned)
This is the **face** of the project, where users will interact with the system.

-   **Type:** Next.js application (TypeScript).
-   **Responsibilities:**
    -   Providing a user-friendly interface for creating intents (e.g., a simple swap form).
    -   Integrating with Solana wallets via the **Wallet Adapter** library for user authentication and transaction signing (if required).
    -   Communicating with the `solver-service` via its REST API to submit intents and receive updates.

### `packages/programs` (Planned)
This directory will contain all on-chain Solana programs written with the Anchor framework.

-   **Type:** Anchor Rust crates.
-   **Responsibilities:**
    -   While the solver's logic is off-chain, we may need on-chain programs for specific tasks that require atomicity or trustlessness.
    -   Examples:
        -   A "bento box" style vault program to hold user funds and allow the solver to execute trades on their behalf without needing the user to sign every transaction.
        -   A multi-instruction dispatcher program that can atomically execute a complex series of CPIs composed by the solver.

## Architectural Patterns & Principles

1.  **Workspace Monorepo:** The entire project is managed as a Cargo workspace. This ensures unified dependency management (`Cargo.lock`), simplified cross-crate development, and streamlined CI processes. All Rust components can be built and tested with a single command (`cargo build --workspace`).

2.  **Core / Binary Separation:** We strictly follow the pattern of separating our core logic (`solver-core`) from the binary that runs it (`solver-service`). This is the most critical architectural decision, providing:
    -   **Testability:** Core algorithms can be unit-tested without needing to mock HTTP requests or a running server.
    -   **Reusability:** The `solver-core` library could be used by other binaries in the future (e.g., a command-line tool, a different type of service) without any changes.
    -   **Clarity:** The `solver-service` crate is lightweight and only concerned with server logic, making it easy to manage.

3.  **Off-Chain Computation:** The heavy lifting of "solving" an intent is done off-chain. This is by design. On-chain programs are expensive, slow, and not suited for complex computations, simulations, or external data fetching. Our on-chain programs will only be used for validation and final atomic execution.

## Technology Stack

-   **Backend:**
    -   **Language:** Rust (Stable toolchain)
    -   **Web Framework:** Axum
    -   **Async Runtime:** Tokio
    -   **Solana Interaction:** `solana-client`, `solana-sdk`
-   **Frontend:**
    -   **Framework:** Next.js
    -   **Language:** TypeScript
    -   **Wallet Integration:** `@solana/wallet-adapter`
    -   **UI:** Tailwind CSS, shadcn/ui
-   **On-Chain:**
    -   **Framework:** Anchor
    -   **Language:** Rust
-   **CI/CD:**
    -   **Platform:** GitHub Actions
-   **Documentation:**
    -   **Framework:** Docusaurus