use anyhow::{Context, Result};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub fn download_file(url: &str, output_path: &PathBuf) -> Result<()> {
    log::info!("Downloading from {} to {:?}", url, output_path);

    let response = reqwest::blocking::get(url)
        .context("Failed to download file")?;

    if !response.status().is_success() {
        anyhow::bail!("Download failed with status: {}", response.status());
    }

    let mut file = fs::File::create(output_path)
        .context("Failed to create output file")?;

    let bytes = response.bytes()
        .context("Failed to read response bytes")?;

    file.write_all(&bytes)
        .context("Failed to write downloaded data")?;

    log::info!("Download completed: {} bytes", bytes.len());
    Ok(())
}

