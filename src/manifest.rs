use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    #[serde(rename = "release_zip_url")]
    pub release_zip_url: String,
    pub sha256: String,
    pub files: Vec<FileEntry>,
    #[serde(default)]
    pub prerequisites: Prerequisites,
    #[serde(rename = "license_check_url")]
    #[serde(default)]
    pub license_check_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Prerequisites {
    #[serde(rename = "windows_version_min")]
    #[serde(default)]
    pub windows_version_min: Option<String>,
    #[serde(rename = "vc_redist")]
    #[serde(default)]
    pub vc_redist: Option<VcRedist>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VcRedist {
    pub required: bool,
    pub url: String,
}

impl Manifest {
    pub fn from_url(url: &str) -> Result<Self> {
        log::info!("Fetching manifest from: {}", url);
        let response = reqwest::blocking::get(url)
            .context("Failed to fetch manifest from URL")?;

        if !response.status().is_success() {
            anyhow::bail!("Manifest fetch failed with status: {}", response.status());
        }

        let text = response.text().context("Failed to read manifest response")?;
        let manifest: Manifest = serde_json::from_str(&text)
            .context("Failed to parse manifest JSON")?;

        manifest.validate()?;
        log::info!("Manifest validated successfully: version {}", manifest.version);

        Ok(manifest)
    }

    pub fn validate(&self) -> Result<()> {
        if self.version.is_empty() {
            anyhow::bail!("Manifest version is empty");
        }

        if self.release_zip_url.is_empty() {
            anyhow::bail!("Manifest release_zip_url is empty");
        }

        if self.sha256.is_empty() {
            anyhow::bail!("Manifest sha256 is empty");
        }

        if self.files.is_empty() {
            anyhow::bail!("Manifest files list is empty");
        }

        for file in &self.files {
            if file.name.is_empty() {
                anyhow::bail!("File entry has empty name");
            }
        }

        Ok(())
    }


    pub fn check_prerequisites(&self) -> Result<()> {
        if let Some(min_version) = &self.prerequisites.windows_version_min {
            log::info!("Checking Windows version requirement: {}", min_version);
            let current_version = get_windows_version()?;
            log::info!("Current Windows version: {}", current_version);
        }

        if let Some(vc_redist) = &self.prerequisites.vc_redist {
            if vc_redist.required {
                log::info!("VC++ Redistributable may be required: {}", vc_redist.url);
            }
        }

        Ok(())
    }
}

fn get_windows_version() -> Result<String> {
    use windows::Win32::System::Registry::*;
    use windows::core::PCSTR;

    unsafe {
        let mut hkey = HKEY::default();
        let result = RegOpenKeyExA(
            HKEY_LOCAL_MACHINE,
            PCSTR(b"SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\0".as_ptr() as *const u8),
            0,
            KEY_READ,
            &mut hkey,
        );

        if result.is_err() {
            return Ok("Unknown".to_string());
        }

        let mut version_size = 256u32;
        let mut version_buffer = vec![0u8; version_size as usize];

        let result = RegQueryValueExA(
            hkey,
            PCSTR(b"CurrentVersion\0".as_ptr() as *const u8),
            None,
            None,
            Some(version_buffer.as_mut_ptr()),
            Some(&mut version_size),
        );

        let _ = RegCloseKey(hkey);

        if result.is_ok() {
            version_buffer.truncate(version_size as usize - 1);
            if let Ok(version) = std::str::from_utf8(&version_buffer) {
                return Ok(version.to_string());
            }
        }

        Ok("Unknown".to_string())
    }
}

