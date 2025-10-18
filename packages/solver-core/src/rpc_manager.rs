use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::Mutex;

// A simple struct to hold the health status of an RPC endpoint.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RpcHealth {
    pub url: String,
    pub is_healthy: bool,
}

// Our main ConnectionManager struct.
pub struct ConnectionManager {
    clients: Vec<RpcClient>,
    health: Arc<Mutex<Vec<bool>>>,
    // AtomicUsize is a thread-safe counter we use for round-robin.
    next_client_index: AtomicUsize,
}

impl ConnectionManager {
    /// Creates a new ConnectionManager from a list of RPC URLs.
    pub fn new(rpc_urls: Vec<String>) -> Self {
        let clients: Vec<RpcClient> = rpc_urls
            .into_iter()
            .map(|url| RpcClient::new(url))
            .collect();

        let health = Arc::new(Mutex::new(vec![true; clients.len()]));
        
        Self {
            clients,
            health,
            next_client_index: AtomicUsize::new(0),
        }
    }

    /// Returns the first healthy RPC client, cycling through them in a round-robin fashion.
    pub async fn get_healthy_client(&self) -> &RpcClient {
        let health_lock = self.health.lock().await;
        let start_index = self.next_client_index.load(Ordering::Relaxed);

        // Loop through clients to find a healthy one.
        for i in 0..self.clients.len() {
            let index = (start_index + i) % self.clients.len();
            if health_lock[index] {
                // Update the index for the next call.
                self.next_client_index.store((index + 1) % self.clients.len(), Ordering::Relaxed);
                return &self.clients[index];
            }
        }

        // If no clients are healthy, just return the next one in the cycle as a fallback.
        let fallback_index = (start_index + 1) % self.clients.len();
        self.next_client_index.store(fallback_index, Ordering::Relaxed);
        &self.clients[start_index]
    }
    
    /// Spawns a background task that periodically checks the health of all RPC endpoints.
    pub fn start_health_checker(self: &Arc<Self>) {
        let self_clone = self.clone();
        tokio::spawn(async move {
            let interval = tokio::time::interval(std::time::Duration::from_secs(30));
            tokio::pin!(interval);
    
            loop {
                interval.tick().await;
                println!("[Health Check] Checking RPC endpoints...");
                let health_checks = self_clone.clients.iter().map(|client| async {
                    client.get_slot().await.is_ok()
                });
    
                let results = futures::future::join_all(health_checks).await;
    
                let mut health_lock = self_clone.health.lock().await;
                *health_lock = results;
                println!("[Health Check] Status: {:?}", *health_lock);
            }
        });
    }

    /// Returns the current health status of all managed RPC endpoints.
    pub async fn get_health_status(&self) -> Vec<RpcHealth> {
        let health_lock = self.health.lock().await;
        self.clients.iter().enumerate().map(|(i, client)| RpcHealth {
            url: client.url(),
            is_healthy: health_lock.get(i).cloned().unwrap_or(false),
        }).collect()
    }
}

