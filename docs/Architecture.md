This is how the project is structured. The main idea is to keep things clean and separated so we can build without breaking stuff.

### The Basic Flow

It's pretty simple. The user's intent goes from the frontend to the backend, which figures out the best way to make it happen on Solana.

```
User's Intent (Frontend)
      |
      v
API Endpoint (Solver Service)
      |
      v
The Actual Logic (Solver Core)
      |
      v
Solana Network
```

### The Pieces

The project lives in a monorepo under the `packages/` directory.

-   **`packages/solver-core` - The Brain.**
    This is a Rust library (`lib.rs`). It contains all the real logic for figuring out intents. It knows nothing about web servers or anything else, which makes it easy to test.

-   **`packages/solver-service` - The Server.**
    A lightweight Rust binary (`main.rs`) that runs the `solver-core`. It starts an Axum web server, exposes API endpoints like `/health`, and handles web requests. That's it.

-   **`packages/frontend` - The UI.**
    A Next.js app where users connect their wallet and tell us what they want to do. It talks to the `solver-service` API.

-   **`packages/programs` - On-Chain Stuff.**
    For any Anchor programs we might need later (like a vault for holding funds). For now, it's a placeholder.

### Why It's Set Up This Way

We're following a couple of key patterns.

1.  **Workspace:** Everything is in a Cargo workspace defined by the root `Cargo.toml`. This keeps dependencies sane and lets you build/test everything at once from the root with `cargo build --workspace`.

2.  **Core vs. Service:** The `solver-core` library is where the thinking happens. The `solver-service` just runs it. **This is the most important pattern here.** It means we can write simple unit tests for the hard parts without needing to spin up a server.

3.  **Off-Chain First:** All the heavy lifting (finding the best route, simulations, etc.) happens off-chain in our Rust code. On-chain programs should be simple and just execute what they're told.

### Tech Stack

-   **Backend:** Rust, Axum, Tokio
-   **Frontend:** Next.js, TypeScript, Tailwind CSS, Solana Wallet Adapter
-   **On-Chain:** Anchor (Rust)
-   **CI/CD:** GitHub Actions