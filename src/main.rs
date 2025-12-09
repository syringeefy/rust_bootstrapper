// syringee made this thx

mod cli;
mod download;
mod install;
mod manifest;
mod shortcut;
mod verify;

use anyhow::Result;
use log::{error, info};
use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};
use std::fs;
use std::io::{self, Write};

use cli::InstallMode;
use install::Installer;

const MANIFEST_URL: &str = "https://raw.githubusercontent.com/syringeefy/Xenith/refs/heads/main/installer.json";

fn setup_logging() -> Result<()> {
    let log_dir = directories::BaseDirs::new()
        .ok_or_else(|| anyhow::anyhow!("Failed to get base directories"))?
        .data_local_dir()
        .join("paradise")
        .join("logs");

    fs::create_dir_all(&log_dir)?;

    let log_file = log_dir.join(format!(
        "bootstrapper_{}.log",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    ));

    let config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .set_target_level(LevelFilter::Error)
        .set_location_level(LevelFilter::Debug)
        .build();

    WriteLogger::init(LevelFilter::Info, config, fs::File::create(log_file)?)?;

    Ok(())
}

fn show_menu() -> Result<InstallMode> {
    println!("paradise bootstrapper");
    println!("1) standard install (appdata)");
    println!("2) custom path install");
    print!("choice: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice = input.trim();

    match choice {
        "1" => Ok(InstallMode::Standard),
        "2" => Ok(InstallMode::Specific),
        _ => {
            println!("invalid choice");
            show_menu()
        }
    }
}

fn get_build_directory() -> Result<std::path::PathBuf> {
    print!("install path: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let path = input.trim();

    if path.is_empty() {
        anyhow::bail!("Directory path cannot be empty");
    }

    let path_buf = std::path::PathBuf::from(path);
    Ok(path_buf)
}

fn main() -> Result<()> {
    setup_logging()?;

    info!("paradise Bootstrapper starting");
    info!("Manifest URL: {}", MANIFEST_URL);

    let mode = show_menu()?;
    let build_dir = if matches!(mode, InstallMode::Specific) {
        Some(get_build_directory()?)
    } else {
        None
    };

    if let Some(ref dir) = build_dir {
        info!("Build directory: {:?}", dir);
    }

    let installer = Installer::new(
        MANIFEST_URL.to_string(),
        mode,
        build_dir,
        false,
        false,
    )?;

    match installer.run() {
        Ok(_) => {
            info!("Installation completed successfully");
            println!("\ninstall complete");
            print!("press enter to exit...");
            io::stdout().flush()?;
            let _ = io::stdin().read_line(&mut String::new());
            Ok(())
        }
        Err(e) => {
            error!("Installation failed: {}", e);
            println!("\ninstall failed: {}", e);
            println!("check logs in %LOCALAPPDATA%\\paradise\\logs");
            print!("press enter to exit...");
            io::stdout().flush()?;
            let _ = io::stdin().read_line(&mut String::new());
            std::process::exit(1);
        }
    }
}

