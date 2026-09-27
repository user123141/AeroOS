//! OCI runtime: pull образов + конвертация в initramfs.
//! Реализовано через reqwest + tar + flate2 (без oci-client).

use anyhow::{bail, Context, Result};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OciManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub config: OciDescriptor,
    pub layers: Vec<OciDescriptor>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OciDescriptor {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub digest: String,
    pub size: u64,
}

pub struct OciRuntime {
    image_dir: PathBuf,
    client: reqwest::Client,
}

impl OciRuntime {
    pub fn new(image_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(image_dir).ok();
        Ok(Self {
            image_dir: image_dir.to_path_buf(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()?,
        })
    }

    /// Скачать OCI-манифест из реестра (публичный).
    pub async fn fetch_manifest(&self, registry: &str, repo: &str, tag: &str) -> Result<OciManifest> {
        let url = format!("https://{}/v2/{}/manifests/{}", registry, repo, tag);
        tracing::info!("Fetching manifest: {}", url);

        let resp = self.client.get(&url)
            .header("Accept", "application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.v2+json")
            .send().await
            .context("HTTP request")?
            .error_for_status()?;
        let manifest: OciManifest = resp.json().await.context("JSON parse")?;
        Ok(manifest)
    }

    /// Скачать и распаковать слои.
    pub async fn pull_layers(&self, registry: &str, repo: &str, manifest: &OciManifest) -> Result<PathBuf> {
        let out_dir = self.image_dir.join(repo.replace('/', "_"));
        std::fs::create_dir_all(&out_dir)?;

        for (i, layer) in manifest.layers.iter().enumerate() {
            let url = format!("https://{}/v2/{}/blobs/{}", registry, repo, layer.digest);
            tracing::info!("Layer {}: {}", i, url);

            let bytes = self.client.get(&url).send().await?
                .error_for_status()?
                .bytes().await?;

            let mut data = &bytes[..];
            // decompress gzip
            let mut decoder = flate2::read::GzDecoder::new(&mut data);
            let mut tar_data = Vec::new();
            std::io::Read::read_to_end(&mut decoder, &mut tar_data)?;

            let mut archive = tar::Archive::new(&tar_data[..]);
            archive.unpack(&out_dir)?;
        }

        tracing::info!("Image extracted to {}", out_dir.display());
        Ok(out_dir)
    }

    /// Конвертировать OCI-образ в initramfs.
    pub fn to_initramfs(&self, image_dir: &Path, out_path: &Path) -> Result<()> {
        let mut builder = CpioNewcBuilder::new();

        for entry in walkdir::WalkDir::new(image_dir) {
            let entry = entry?;
            let rel = entry.path().strip_prefix(image_dir)?;
            let name = format!("/{}", rel.display());

            if entry.file_type().is_dir() {
                builder.add_dir(&name)?;
            } else if entry.file_type().is_file() {
                let data = std::fs::read(entry.path())?;
                builder.add_file(&name, &data)?;
            }
        }

        builder.add_file("/init", b"#!/bin/sh\nmount -t proc proc /proc\nmount -t sysfs sys /sys\nexec /bin/sh\n")?;
        builder.finish(out_path)?;
        tracing::info!("Initramfs written: {}", out_path.display());
        Ok(())
    }
}

struct CpioNewcBuilder {
    entries: Vec<(String, Vec<u8>, bool)>,
    ino: u32,
}

impl CpioNewcBuilder {
    fn new() -> Self { Self { entries: Vec::new(), ino: 1 } }
    fn add_dir(&mut self, name: &str) -> Result<()> {
        self.entries.push((name.to_string(), Vec::new(), true));
        self.ino += 1;
        Ok(())
    }
    fn add_file(&mut self, name: &str, data: &[u8]) -> Result<()> {
        self.entries.push((name.to_string(), data.to_vec(), false));
        self.ino += 1;
        Ok(())
    }
    fn finish(self, out_path: &Path) -> Result<()> {
        let mut file = std::fs::File::create(out_path)?;
        for (name, data, is_dir) in &self.entries {
            write_newc_entry(&mut file, name, data, *is_dir)?;
        }
        write_newc_entry(&mut file, "TRAILER!!!", &[], false)?;
        file.sync_all()?;
        Ok(())
    }
}

fn write_newc_entry<W: Write>(w: &mut W, name: &str, data: &[u8], is_dir: bool) -> Result<()> {
    let mode = if is_dir { 0o040755 } else { 0o100644 };
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len() + 1;

    let header = format!(
        "070701{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}{:08X}",
        1, mode, 0, 0, 1, 0, data.len(), 0, 0, 0, 0, name_len, 0
    );
    w.write_all(header.as_bytes())?;
    w.write_all(name_bytes)?;
    w.write_all(&[0])?;
    let pad = (4 - (110 + name_len) % 4) % 4;
    w.write_all(&vec![0u8; pad])?;

    if !is_dir && !data.is_empty() {
        w.write_all(data)?;
        let pad = (4 - data.len() % 4) % 4;
        w.write_all(&vec![0u8; pad])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_manifest() {
        let json = r#"{
            "schemaVersion": 2,
            "mediaType": "application/vnd.oci.image.manifest.v1+json",
            "config": {"mediaType":"x", "digest":"sha256:abc", "size":123},
            "layers": []
        }"#;
        let m: OciManifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.schema_version, 2);
        assert_eq!(m.media_type, "application/vnd.oci.image.manifest.v1+json");
    }

    #[test]
    fn test_cpio_builder() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.cpio");
        let mut b = CpioNewcBuilder::new();
        b.add_dir("/test")?;
        b.add_file("/test/hello.txt", b"Hello, AeroOS!")?;
        b.finish(&path)?;
        assert!(path.exists());
        assert!(std::fs::metadata(&path).unwrap().len() > 0);
    }
}