use async_trait::async_trait;
use forge_sdk::error::ForgeError;

#[async_trait]
pub trait SecretStore: Send + Sync {
    async fn get(&self, key: &str) -> Result<String, ForgeError>;
}

#[async_trait]
pub trait CloudStorage: Send + Sync {
    async fn put(&self, key: &str, data: &[u8]) -> Result<(), ForgeError>;
    async fn get(&self, key: &str) -> Result<Vec<u8>, ForgeError>;
}
