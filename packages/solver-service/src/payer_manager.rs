use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, SeedDerivable, Signer},
};
use solver_core::rpc_manager::ConnectionManager;
use std::env;
use std::sync::Arc;

pub struct PayerManager {
    keypair: Keypair,
    connection_manager: Arc<ConnectionManager>,
}

impl PayerManager {
    pub fn from_env(connection_manager: Arc<ConnectionManager>) -> Self {
        let keypair = if let Ok(seed_phrase) = env::var("SEED_PHRASE") {
            let mnemonic = bip39::Mnemonic::parse(&seed_phrase)
                .expect("FATAL: Invalid SEED_PHRASE. Must be valid BIP39 mnemonic.");
            let seed = mnemonic.to_seed("");
            Keypair::from_seed(&seed[..32])
                .expect("FATAL: Failed to create keypair from seed phrase.")
        } else if let Ok(private_key_b58) = env::var("PRIVATE_KEY") {
            Keypair::from_base58_string(&private_key_b58)
        } else {
            panic!("FATAL: Either SEED_PHRASE or PRIVATE_KEY environment variable must be set.");
        };

        tracing::info!(pubkey = %keypair.pubkey(), "Loaded payer wallet");

        Self {
            keypair,
            connection_manager,
        }
    }

    pub fn public_key(&self) -> Pubkey {
        self.keypair.pubkey()
    }

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

    async fn check_balance(&self) {
        const LOW_BALANCE_THRESHOLD_SOL: f64 = 0.1;

        let client = self.connection_manager.get_healthy_client().await;
        match client.get_balance(&self.keypair.pubkey()).await {
            Ok(balance_lamports) => {
                let balance_sol =
                    balance_lamports as f64 / solana_sdk::native_token::LAMPORTS_PER_SOL as f64;
                tracing::info!(
                    balance_sol = format!("{:.4}", balance_sol),
                    "Payer wallet balance"
                );

                if balance_sol < LOW_BALANCE_THRESHOLD_SOL {
                    tracing::warn!(
                        balance_sol = format!("{:.4}", balance_sol),
                        "Payer balance is low! Please refill."
                    );
                }
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to check payer balance");
            }
        }
    }
}
