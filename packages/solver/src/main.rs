use tokio;

async fn fetch_priority_fees() -> Result<u64, ()> {
    Ok(100_000)
}

#[tokio::main]
async fn main() {
    println!("Starting Solana Intent Solver...");
    match fetch_priority_fees().await {
        Ok(fee) => println!("Current estimated priority fee: {} micro-lamports", fee),
        Err(_) => eprintln!("Failed to fetch priority fees."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_priority_fees_mock() {
        let fee = fetch_priority_fees().await;
        assert!(fee.is_ok());
        assert_eq!(fee.unwrap(), 100_000);
    }
}
