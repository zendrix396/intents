use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use solver_core::rpc_manager::ConnectionManager;
use std::env;
use std::sync::Arc;

// Holds the keypair for paying transaction fees
pub struct PayerManager {
    keypair: Keypair,
    connection_manager: Arc<ConnectionManager>,
}

impl PayerManager {
    /// Creates a new PayerManager by loading a private key from the environment.
    pub fn from_env(connection_manager: Arc<ConnectionManager>) -> Self {
        let private_key_b58 =
            env::var("PRIVATE_KEY").expect("FATAL: PRIVATE_KEY environment variable not set.");

        let keypair = Keypair::from_base58_string(&private_key_b58);

        println!("[Payer Manager] Loaded payer wallet: {}", keypair.pubkey());

        Self {
            keypair,
            connection_manager,
        }
    }

    /// Returns the payer's public key.
    pub fn public_key(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    /// Spawns a background task to periodically check the payer's balance.
    pub fn start_balance_monitor(self: &Arc<Self>) {
        let self_clone = self.clone();
        tokio::spawn(async move {
            let interval = tokio::time::interval(std::time::Duration::from_secs(60));
            tokio::pin!(interval);

            loop {
                interval.as_mut().tick().await;
                self_clone.check_balance().await;
            }
        });
    }

    /// Checks the current balance of the payer wallet and logs a warning if it's low.
    async fn check_balance(&self) {
        const LOW_BALANCE_THRESHOLD_SOL: f64 = 0.1;

        let client = self.connection_manager.get_healthy_client().await;
        match client.get_balance(&self.keypair.pubkey()).await {
            Ok(balance_lamports) => {
                let balance_sol =
                    balance_lamports as f64 / solana_sdk::native_token::LAMPORTS_PER_SOL as f64;
                println!("[Payer Manager] Current balance: {balance_sol:.4} SOL");

                if balance_sol < LOW_BALANCE_THRESHOLD_SOL {
                    eprintln!(
                        "[Payer Manager] WARNING: Payer balance is low ({balance_sol:.4} SOL). Please refill."
                    );
                }
            }
            Err(e) => {
                eprintln!("[Payer Manager] Failed to check payer balance: {e}");
            }
        }
    }
}
