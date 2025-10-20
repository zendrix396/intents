use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, SeedDerivable, Signer},
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
    /// Creates a new PayerManager by loading a keypair from the environment.
    /// Supports either SEED_PHRASE or PRIVATE_KEY env vars.
    pub fn from_env(connection_manager: Arc<ConnectionManager>) -> Self {
        let keypair = if let Ok(seed_phrase) = env::var("SEED_PHRASE") {
            // Load from seed phrase
            let mnemonic = bip39::Mnemonic::parse(&seed_phrase)
                .expect("FATAL: Invalid SEED_PHRASE. Must be valid BIP39 mnemonic.");
            let seed = mnemonic.to_seed("");
            Keypair::from_seed(&seed[..32])
                .expect("FATAL: Failed to create keypair from seed phrase.")
        } else if let Ok(private_key_b58) = env::var("PRIVATE_KEY") {
            // Load from base58 private key
            Keypair::from_base58_string(&private_key_b58)
        } else {
            panic!("FATAL: Either SEED_PHRASE or PRIVATE_KEY environment variable must be set.");
        };

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

    /// Returns a reference to the keypair.
    /// This is needed for signing transactions.
    pub fn get_keypair(&self) -> &Keypair {
        &self.keypair
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
