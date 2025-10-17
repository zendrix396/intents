# An Intent-Centric Layer for Solana

This document outlines what we're building, why it's cool, and how we plan to get there.

## What is This Thing? (The TL;DR)

Right now, using DeFi on Solana means telling the blockchain *exactly* what to do: "Swap X for Y on Raydium with 0.5% slippage."

We're building an **intent-centric execution layer**.

Instead of giving detailed instructions, a user just states their goal (their "intent"):

> "I want to swap 1 SOL and get the most USDC possible."

Our system's job is to figure out the **best way** to make that happen and then do it for them.

## The Interface: How Users Interact

The power of an intent-based system is its flexibility. The core solver logic is completely separate from how a user submits their intent. We plan to support multiple interfaces:

-   **REST API:** The primary interface for developers and our own frontend. Other dApps, wallets, and services will be able to integrate with our solver by calling a simple API endpoint.
-   **Frontend Web App:** A clean, user-friendly interface for end-users to connect their wallet and create intents through a simple form.
-   **Telegram/Discord Bots (Future):** Users will be able to type natural language commands like `/swap 1 sol for usdc` in a chat, and the bot will guide them through the process.

## Who Is This For?

1.  **Users:** Who want better prices and a simpler experience without having to manually compare DEXs or route swaps.
2.  **Developers:** Who are building applications (wallets, dApps, bots) and want to offer their users the best execution without building a complex routing engine themselves.
3.  **Sophisticated Traders:** Who want to express complex intents, like "Sell my 100 BONK for SOL, but only if the price is above $X, and do it over the next hour to minimize impact."

## How It Works: The Core Components

### 1. The Intent Submission API

This is the front door. It will be a simple REST API running in our `solver-service`.

-   **Endpoint:** `POST /intents`
-   **Payload:** A JSON object describing the user's goal.
    ```json
    {
      "type": "MAX_RETURN_SWAP",
      "inputMint": "So11111111111111111111111111111111111111112",
      "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
      "inputAmount": 1000000000
    }
    ```

### 2. The Solver (`solver-core`)

This is the heart of the system. When the API gets an intent, it passes it to the solver. The solver's job is to find the most optimal execution path.

**How it finds the "best" path:**

-   **Data Gathering:** It will constantly monitor the Solana network, pulling data from:
    -   **Liquidity Pools:** Raydium, Orca, Jupiter, etc.
    -   **Oracles:** Pyth and Switchboard for real-time asset prices.
    -   **Network Fees:** Jito's bundle endpoint and Solana's `getRecentPriorityFees` to understand current transaction costs.

-   **Heuristics & Solvers:** This is where the magic happens. We'll start simple and build up.
    1.  **Simple Heuristics (Phase 1):** At first, the solver will use basic rules. For a swap, it might just query the Jupiter API for the best route and use that. This is our MVP.
    2.  **Custom Routing (Phase 2):** We'll build our own routing logic to find paths Jupiter might miss, like splitting a trade across multiple DEXs.
    3.  **ML-Powered Solvers (Phase 3):** This is the end-game. We'll use the `services` package (placeholder for a Python/FastAPI service) to run more advanced models. The Rust solver will send intent data to this service, which could use machine learning to predict price impact or find complex arbitrage opportunities (e.g., A -> B -> C -> A) that can be bundled with the user's trade to give them an even better price.

-   **The Output:** The solver's final output is a ready-to-send, fully constructed Solana transaction (or a series of them).

### 3. The Executor / Relayer

Once the solver creates the transaction, something has to sign it and send it to the network.

-   The `solver-service` will act as the relayer. It will manage a secure keypair (the "hot wallet") to pay for transaction fees.
-   It will use a robust transaction-sending function that includes **priority fees** to ensure our transactions are processed quickly, even when the network is busy.

### 4. The Frontend

The user interface will be a clean, simple web app where users can:

1.  **Connect their wallet.**
2.  **Express an intent** through a simple form (e.g., "I want to swap...").
3.  **Approve the final transaction.** For self-custody, the user will sign the transaction that the solver proposes. In a more advanced version, they might delegate funds to a secure on-chain vault that the solver can use to execute trades on their behalf.

## Tech Stack Summary

-   **Solver & API:** Rust (Tokio, Axum)
-   **Advanced Heuristics (Future):** Python (FastAPI, scikit-learn/PyTorch)
-   **Frontend:** Next.js (TypeScript, React)
-   **On-Chain (Future):** Anchor (Rust)
-   **Documentation:** MDX within the Next.js app.

This approach gives us the performance of Rust for the core, real-time components, the power of Python for complex offline analysis, and the rapid development of Next.js for the user interface.