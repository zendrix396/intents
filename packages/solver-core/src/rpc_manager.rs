use solana_client::nonblocking::rpc_client::RpcClient;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize)]
pub struct RpcHealth {
    pub url: String,
    pub is_healthy: bool,
}

pub struct ConnectionManager {
    clients: Vec<RpcClient>,
    health: Arc<Mutex<Vec<bool>>>,
    next_client_index: AtomicUsize,
}

impl ConnectionManager {
    pub fn new(rpc_urls: Vec<String>) -> Self {
        let clients: Vec<RpcClient> = rpc_urls.into_iter().map(RpcClient::new).collect();

        let health = Arc::new(Mutex::new(vec![true; clients.len()]));

        tracing::info!("ConnectionManager initialized with {} RPC endpoints", clients.len());

        Self {
            clients,
            health,
            next_client_index: AtomicUsize::new(0),
        }
    }

    pub async fn get_healthy_client(&self) -> &RpcClient {
        let health_lock = self.health.lock().await;
        let start_index = self.next_client_index.load(Ordering::Relaxed);

        for i in 0..self.clients.len() {
            let index = (start_index + i) % self.clients.len();
            if health_lock[index] {
                self.next_client_index
                    .store((index + 1) % self.clients.len(), Ordering::Relaxed);
                return &self.clients[index];
            }
        }

        tracing::warn!("No healthy RPC clients found, using fallback");
        let fallback_index = (start_index + 1) % self.clients.len();
        self.next_client_index
            .store(fallback_index, Ordering::Relaxed);
        &self.clients[start_index]
    }

    pub fn start_health_checker(self: &Arc<Self>) {
        let self_clone = self.clone();
        tokio::spawn(async move {
            let interval = tokio::time::interval(std::time::Duration::from_secs(30));
            tokio::pin!(interval);

            loop {
                interval.tick().await;
                tracing::info!("Checking RPC endpoint health...");
                let health_checks = self_clone
                    .clients
                    .iter()
                    .map(|client| async { client.get_slot().await.is_ok() });

                let results = futures::future::join_all(health_checks).await;

                let mut health_lock = self_clone.health.lock().await;
                *health_lock = results;
                tracing::debug!("RPC health status: {:?}", *health_lock);
            }
        });
    }

    pub async fn get_health_status(&self) -> Vec<RpcHealth> {
        let health_lock = self.health.lock().await;
        self.clients
            .iter()
            .enumerate()
            .map(|(i, client)| RpcHealth {
                url: client.url(),
                is_healthy: health_lock.get(i).cloned().unwrap_or(false),
            })
            .collect()
    }
}
