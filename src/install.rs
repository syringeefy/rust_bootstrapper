use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::InstallMode;
use crate::download::download_file;
use crate::manifest::Manifest;
use crate::shortcut::create_shortcut;
use crate::verify::verify_sha256;
use atomic::AtomicInstaller;

pub struct Installer {
    manifest_url: String,
    mode: InstallMode,
    build_dir: Option<PathBuf>,
    dry_run: bool,
    no_shortcut: bool,
}

impl Installer {
    pub fn new(
        manifest_url: String,
        mode: InstallMode,
        build_dir: Option<PathBuf>,
        dry_run: bool,
        no_shortcut: bool,
    ) -> Result<Self> {
        if matches!(mode, InstallMode::Specific) && build_dir.is_none() {
            anyhow::bail!("Build directory is required for specific mode");
        }

        Ok(Self {
            manifest_url,
            mode,
            build_dir,
            dry_run,
            no_shortcut,
        })
    }

    pub fn run(&self) -> Result<()> {
        log::info!("Starting installation process");

        let manifest = Manifest::from_url(&self.manifest_url)?;
        manifest.check_prerequisites()?;

        let install_dir = self.get_install_directory()?;
        log::info!("Install directory: {:?}", install_dir);

        if self.dry_run {
            log::info!("DRY RUN: Would download from {}", manifest.release_zip_url);
            log::info!("DRY RUN: Would install to {:?}", install_dir);
            return Ok(());
        }

        let temp_dir = tempfile::tempdir()
            .context("Failed to create temporary directory")?;

        let zip_path = temp_dir.path().join("release.zip");
        download_file(&manifest.release_zip_url, &zip_path)?;

        verify_sha256(&zip_path, &manifest.sha256)
            .context("ZIP file SHA256 verification failed")?
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("ZIP file integrity check failed"))?;

        let extract_dir = temp_dir.path().join("extracted");
        fs::create_dir_all(&extract_dir)?;
        self.extract_zip(&zip_path, &extract_dir)?;

        self.verify_extracted_files(&extract_dir, &manifest)?;

        let atomic_installer = AtomicInstaller::new(&install_dir)?;
        atomic_installer.install(&extract_dir)?;

        if !self.no_shortcut {
            self.create_shortcuts(&install_dir)?;
        }

        log::info!("Installation completed successfully");
        Ok(())
    }

    fn get_install_directory(&self) -> Result<PathBuf> {
        match &self.mode {
            InstallMode::Standard => {
                let base = directories::BaseDirs::new()
                    .ok_or_else(|| anyhow::anyhow!("Failed to get base directories"))?
                    .data_local_dir()
                    .join("paradise")
                    .join("appfolder");
                Ok(base)
            }
            InstallMode::Specific => {
                let dir = self
                    .build_dir
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Build directory not specified"))?;
                Ok(dir.clone())
            }
        }
    }

    fn extract_zip(&self, zip_path: &Path, extract_dir: &Path) -> Result<()> {
        log::info!("Extracting ZIP to {:?}", extract_dir);

        let file = fs::File::open(zip_path)
            .context("Failed to open ZIP file")?;

        let mut archive = zip::ZipArchive::new(file)
            .context("Failed to read ZIP archive")?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .context("Failed to read file from ZIP")?;

            let outpath = extract_dir.join(file.mangled_name());

            if file.is_dir() {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)?;
                }
                let mut outfile = fs::File::create(&outpath)
                    .context("Failed to create extracted file")?;
                std::io::copy(&mut file, &mut outfile)
                    .context("Failed to write extracted file")?;
            }
        }

        log::info!("ZIP extraction completed");
        Ok(())
    }

    fn verify_extracted_files(&self, extract_dir: &Path, manifest: &Manifest) -> Result<()> {
        log::info!("Checking extracted files");

        for file_entry in &manifest.files {
            let file_path = extract_dir.join(&file_entry.name);
            
            if !file_path.exists() {
                anyhow::bail!("Required file not found in archive: {}", file_entry.name);
            }
        }

        log::info!("All required files found");
        Ok(())
    }

    fn create_shortcuts(&self, install_dir: &Path) -> Result<()> {
        let exe_path = install_dir.join("paradise.exe");
        
        if !exe_path.exists() {
            log::warn!("paradise.exe not found, skipping shortcut creation");
            return Ok(());
        }

        match &self.mode {
            InstallMode::Standard => {
                let desktop = directories::UserDirs::new()
                    .and_then(|d| d.desktop_dir().map(|p| p.to_path_buf()))
                    .ok_or_else(|| anyhow::anyhow!("Failed to get desktop directory"))?;

                let shortcut_path = desktop.join("paradise.lnk");
                create_shortcut(&exe_path, &shortcut_path)?;
                log::info!("Desktop shortcut created: {:?}", shortcut_path);
            }
            InstallMode::Specific => {
                let shortcut_path = install_dir.join("paradise.lnk");
                create_shortcut(&exe_path, &shortcut_path)?;
                log::info!("Shortcut created in build directory: {:?}", shortcut_path);
            }
        }

        Ok(())
    }
}

mod atomic {
    use anyhow::{Context, Result};
    use std::fs;
    use std::path::{Path, PathBuf};

    pub struct AtomicInstaller {
        target_dir: PathBuf,
        backup_dir: Option<PathBuf>,
    }

    impl AtomicInstaller {
        pub fn new(target_dir: &Path) -> Result<Self> {
            let target_dir = target_dir.to_path_buf();
            let backup_dir = if target_dir.exists() {
                let backup = target_dir.with_extension("backup");
                Some(backup)
            } else {
                None
            };

            Ok(Self {
                target_dir,
                backup_dir,
            })
        }

        pub fn install(&self, source_dir: &Path) -> Result<()> {
            log::info!("Performing atomic installation to {:?}", self.target_dir);

            if let Some(ref backup) = self.backup_dir {
                log::info!("Backing up existing installation to {:?}", backup);
                if backup.exists() {
                    fs::remove_dir_all(backup)
                        .context("Failed to remove old backup")?;
                }
                fs::rename(&self.target_dir, backup)
                    .context("Failed to create backup")?;
            }

            if let Some(parent) = self.target_dir.parent() {
                fs::create_dir_all(parent)
                    .context("Failed to create parent directory")?;
            }

            fs::rename(source_dir, &self.target_dir)
                .or_else(|_| {
                    fs::create_dir_all(&self.target_dir)?;
                    copy_dir_all(source_dir, &self.target_dir)
                })
                .context("Failed to move/copy installation directory")?;

            log::info!("Atomic installation completed successfully");
            Ok(())
        }
    }

    fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let ty = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if ty.is_dir() {
                copy_dir_all(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }
        Ok(())
    }
}

