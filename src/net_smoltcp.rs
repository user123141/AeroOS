//! User-mode networking via smoltcp.
//! Избавляет от необходимости прав администратора для сети.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SmoltcpBackend {
    // TODO: реализация
}

impl SmoltcpBackend {
    pub fn new() -> Result<Self> {
        tracing::info!("smoltcp backend initialized (user-mode, no admin needed)");
        Ok(Self {})
    }

    pub async fn run(&self) -> Result<()> {
        // TODO: основной цикл обработки пакетов
        Ok(())
    }
}