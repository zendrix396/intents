# How to Contribute

Want to help out? Awesome. Here’s how to get started.

## Getting Set Up

1.  **Clone the repo.**
2.  **Install the tools:** You'll need the latest stable Rust and Node.js.
3.  **Build everything:** From the root of the project, run:
    ```bash
    cargo build --workspace
    ```
    This command builds both the `solver-core` and `solver-service` crates.

4.  **Run tests:** Make sure everything is working before you start changing things.
    ```bash
    cargo test --workspace
    ```

## The Codebase

The project is a Cargo workspace. The important parts are in `packages/`:

-   `solver-core`: This is a Rust library. **All the real logic goes here.** It's just functions and structs. No web server, no runtime stuff.
-   `solver-service`: This is a Rust binary. It runs our code. It starts a web server (Axum) and calls the functions from `solver-core`.
-   `frontend`: A standard Next.js app for the UI.

**The main rule:** Keep the logic in `solver-core` and the "running" part in `solver-service`. This lets us test the important stuff without mocking a web server.

## Making a Pull Request

1.  **Grab an issue:** Find an open issue and get assigned to it. Or, open a new one to discuss your idea first.
2.  **Create a branch:** Name it something like `feature/add-swap-logic`.
3.  **Write your code.** Keep the changes focused on the issue you're solving.
4.  **Format your code:** Before you commit, run `cargo fmt` from the root. Our CI will fail if you don't.
5.  **Make sure tests pass:** `cargo test --workspace`.
6.  **Open a PR:**
    -   Keep the description short and sweet.
    -   Link the issue you're fixing by adding `Fixes #<issue-number>` in the PR description.
    -   Make sure your branch is up to date with `main`.

That's pretty much it. Keep PRs small and focused.