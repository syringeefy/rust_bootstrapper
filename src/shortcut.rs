use anyhow::{Context, Result};
use std::path::Path;
use windows::core::*;
use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::*;

pub fn create_shortcut(exe_path: &Path, shortcut_path: &Path) -> Result<()> {
    log::info!(
        "Creating shortcut: {:?} -> {:?}",
        shortcut_path,
        exe_path
    );

    unsafe {
        CoInitialize(None)
            .ok()
            .context("Failed to initialize COM")?;

        let clsid = windows::core::GUID::from_u128(0x00021401_0000_0000_C000_000000000046);

        let shell_link: IShellLinkW = CoCreateInstance(&clsid, None, CLSCTX_INPROC_SERVER)
            .context("Failed to create IShellLink instance")?;

        let exe_path_hstring = HSTRING::from(
            exe_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid exe path"))?
        );

        shell_link
            .SetPath(&exe_path_hstring)
            .ok()
            .context("Failed to set shortcut path")?;

        let work_dir = exe_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Exe path has no parent"))?;

        let work_dir_hstring = HSTRING::from(
            work_dir
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid work directory path"))?
        );

        shell_link
            .SetWorkingDirectory(&work_dir_hstring)
            .ok()
            .context("Failed to set working directory")?;

        let persist_file: IPersistFile = shell_link.cast()
            .context("Failed to get IPersistFile interface")?;

        let shortcut_path_hstring = HSTRING::from(
            shortcut_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid shortcut path"))?
        );

        persist_file
            .Save(&shortcut_path_hstring, true)
            .ok()
            .context("Failed to save shortcut")?;

        CoUninitialize();
    }

    log::info!("Shortcut created successfully");
    Ok(())
}

