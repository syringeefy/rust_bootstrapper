use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub fn verify_sha256(file_path: &Path, expected_hash: &str) -> Result<bool> {
    log::info!("Verifying SHA256 for {:?}", file_path);

    let file_data = fs::read(file_path)
        .context("Failed to read file for verification")?;

    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let computed_hash = hex::encode(hasher.finalize());

    let matches = computed_hash.to_lowercase() == expected_hash.to_lowercase();
    
    if matches {
        log::info!("SHA256 verification passed for {:?}", file_path);
    } else {
        log::warn!(
            "SHA256 verification failed for {:?}: expected {}, got {}",
            file_path,
            expected_hash,
            computed_hash
        );
    }

    Ok(matches)
}

pub fn compute_sha256(file_path: &Path) -> Result<String> {
    let file_data = fs::read(file_path)
        .context("Failed to read file for hash computation")?;

    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    Ok(hex::encode(hasher.finalize()))
}

