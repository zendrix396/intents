pub async fn fetch_priority_fees() -> Result<u64, ()> {
    Ok(100_000)
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
