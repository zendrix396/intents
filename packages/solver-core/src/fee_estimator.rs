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
                println!("[Fee Monitor] Fetching recent priority fees...");
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
                eprintln!("[Fee Monitor] Failed to fetch fees: {e}");
                vec![]
            }
        };

        if !recent_fees.is_empty() {
            let mut fees_lock = self.recent_fees.lock().await;
            *fees_lock = recent_fees;
            println!("[Fee Monitor] Updated fees cache.");
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
