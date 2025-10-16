use solver_core::fetch_priority_fees; // Import the function from our new library

#[tokio::main]
async fn main() {
    println!("Starting Solana Intent Solver Service...");

    // Call the function from the solver-core library
    match fetch_priority_fees().await {
        Ok(fee) => println!("Current estimated priority fee: {} micro-lamports", fee),
        Err(_) => eprintln!("Failed to fetch priority fees."),
    }
}

// The tests have been moved to solver-core/src/lib.rs, so this section is no longer needed here.
