// barq-vault: barq-proto
// Generated protobuf code lives in src/generated/

/// Generated tonic code for barq_vault.proto.
pub mod barqvault {
    pub mod v1 {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/generated/barqvault.v1.rs"
        ));
    }
}

pub use barqvault::v1::barq_vault_server::BarqVault;
pub use barqvault::v1::barq_vault_server::BarqVaultServer;
pub use barqvault::v1::barq_vault_client::BarqVaultClient;


pub mod convert;

#[cfg(test)]
mod tests;
