use crate::rpc_manager::ConnectionManager;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct FeeEstimator {
    connection_manager: Arc<ConnectionManager>,
    recent_fees: Arc<Mutex<Vec<u64>>>,
}

impl FeeEstimator {
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        Self {
            connection_manager,
            recent_fees: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn start_fee_monitor(self: &Arc<Self>) {
        let self_clone = self.clone();
        tokio::spawn(async move {
            let interval = tokio::time::interval(std::time::Duration::from_secs(15));
            tokio::pin!(interval);

            loop {
                interval.tick().await;
                tracing::info!("Fetching recent priority fees...");
                self_clone.fetch_and_update_fees().await;
            }
        });
    }

    async fn fetch_and_update_fees(&self) {
        let client = self.connection_manager.get_healthy_client().await;

        let recent_fees = match client.get_recent_prioritization_fees(&[]).await {
            Ok(fees) => fees
                .into_iter()
                .take(20)
                .map(|fee| fee.prioritization_fee)
                .collect(),
            Err(e) => {
                tracing::warn!("Failed to fetch fees: {}", e);
                vec![]
            }
        };

        if !recent_fees.is_empty() {
            let mut fees_lock = self.recent_fees.lock().await;
            *fees_lock = recent_fees;
            tracing::debug!("Updated fees cache with {} entries", fees_lock.len());
        }
    }

    pub async fn get_priority_fee_for_level(&self, level: &str) -> u64 {
        let fees_lock = self.recent_fees.lock().await;
        if fees_lock.is_empty() {
            return 5_000;
        }

        let mut fees = fees_lock.clone();
        fees.sort_unstable();

        let percentile = match level {
            "low" => 0.25,
            "medium" => 0.50,
            "high" => 0.75,
            "very-high" => 0.95,
            _ => 0.50,
        };

        calculate_percentile(&fees, percentile)
    }
}

fn calculate_percentile(sorted_data: &[u64], percentile: f64) -> u64 {
    if sorted_data.is_empty() {
        return 0;
    }
    let index = (percentile * (sorted_data.len() - 1) as f64).round() as usize;
    sorted_data[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_percentile_empty() {
        assert_eq!(calculate_percentile(&[], 0.5), 0);
    }

    #[test]
    fn test_calculate_percentile_single() {
        assert_eq!(calculate_percentile(&[10], 0.5), 10);
    }

    #[test]
    fn test_calculate_percentile_low() {
        let data = vec![10, 20, 30, 40, 50];
        assert_eq!(calculate_percentile(&data, 0.25), 20);
    }

    #[test]
    fn test_calculate_percentile_medium() {
        let data = vec![10, 20, 30, 40, 50];
        assert_eq!(calculate_percentile(&data, 0.50), 30);
    }

    #[test]
    fn test_calculate_percentile_high() {
        let data = vec![10, 20, 30, 40, 50];
        assert_eq!(calculate_percentile(&data, 0.75), 40);
    }

    #[test]
    fn test_calculate_percentile_very_high() {
        let data = vec![10, 20, 30, 40, 50];
        assert_eq!(calculate_percentile(&data, 0.95), 50);
    }

    #[test]
    fn test_calculate_percentile_two_elements() {
        let data = vec![100, 200];
        // index = (0.0 * 1).round() = 0 -> 100
        assert_eq!(calculate_percentile(&data, 0.0), 100);
        // index = (0.5 * 1).round() = 1 (rounds up) -> 200
        assert_eq!(calculate_percentile(&data, 0.5), 200);
        // index = (1.0 * 1).round() = 1 -> 200
        assert_eq!(calculate_percentile(&data, 1.0), 200);
    }

    #[test]
    fn test_calculate_percentile_unsorted_input() {
        let data = vec![50, 10, 30, 20, 40];
        let mut sorted = data.clone();
        sorted.sort_unstable();
        assert_eq!(calculate_percentile(&sorted, 0.5), 30);
    }

    #[test]
    fn test_get_priority_fee_level_mapping() {
        let data = vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        let mut sorted = data;
        sorted.sort_unstable();

        // index = (0.25 * 9).round() = 2 -> 30
        assert_eq!(calculate_percentile(&sorted, 0.25), 30);
        // index = (0.50 * 9).round() = 5 -> 60
        assert_eq!(calculate_percentile(&sorted, 0.50), 60);
        // index = (0.75 * 9).round() = 7 -> 80
        assert_eq!(calculate_percentile(&sorted, 0.75), 80);
        // index = (0.95 * 9).round() = 9 -> 100
        assert_eq!(calculate_percentile(&sorted, 0.95), 100);
    }
}
