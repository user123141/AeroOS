//! AeroRegistry — клиент для микро-образов AeroOS.
//! Реальные HTTP-запросы через `reqwest`.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

const REGISTRY_URL: &str = "https://registry.aeroos.dev";

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ImageManifest {
    pub name: String,
    pub version: String,
    pub size_mb: u64,
    pub sha256: String,
    pub url: String,
}

pub struct AeroRegistry {
    base_url: String,
    cache_dir: PathBuf,
    client: reqwest::Client,
}

impl AeroRegistry {
    pub fn new() -> Result<Self> {
        let cache = dirs_cache()?;
        std::fs::create_dir_all(&cache).ok();
        Ok(Self {
            base_url: REGISTRY_URL.into(),
            cache_dir: cache,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
        })
    }

    pub fn list(&self) -> Vec<&'static str> {
        vec!["alpine-mini", "ubuntu-mini", "arch-mini"]
    }

    pub async fn pull(&self, name: &str, out_path: &Path) -> Result<()> {
        let url = format!("{}/images/{}.json", self.base_url, name);
        tracing::info!("Fetching manifest: {}", url);

        let manifest: ImageManifest = self.client.get(&url).send().await
            .context("HTTP request")?
            .error_for_status()?
            .json().await
            .context("JSON parse")?;

        tracing::info!("Image '{}' v{}, {} MB", manifest.name, manifest.version, manifest.size_mb);

        let cached = self.cache_dir.join(format!("{}-{}.tar.gz", manifest.name, manifest.version));
        if !cached.exists() {
            let bytes = self.client.get(&manifest.url).send().await?
                .error_for_status()?
                .bytes().await?;
            std::fs::write(&cached, &bytes)?;
            let actual_sha = sha256_file(&cached)?;
            if actual_sha != manifest.sha256 {
                bail!("SHA-256 mismatch");
            }
        }
        if let Some(parent) = out_path.parent() {
            if parent.exists() { std::fs::copy(&cached, out_path)?; }
        }
        Ok(())
    }

    pub fn cache_dir(&self) -> &Path { &self.cache_dir }
}

fn dirs_cache() -> Result<PathBuf> {
    let base = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME"))?;
    Ok(PathBuf::from(base).join(".aeroos").join("images"))
}

fn sha256_file(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let data = std::fs::read(path)?;
    let mut h = Sha256::new();
    h.update(&data);
    Ok(hex::encode(h.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_list() {
        let r = AeroRegistry::new().unwrap();
        assert!(r.list().contains(&"alpine-mini"));
    }
}