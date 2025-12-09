Here’s a **cleaner, more polished, more minimal GitHub-ready README**, including a short explanation on how to generate the SHA-256 hash for your release ZIP on Windows, Linux, and macOS.

---

# paradise bootstrapper

This is a simple bootstrapper written in rust by me while I was learning rust. Sorry for shit code :P

---

## Build

```bash
cargo build --release
```

Update the manifest URL inside `src/main.rs`:

```rust
const MANIFEST_URL: &str = "https://raw.githubusercontent.com/syringeefy/Xenith/refs/heads/main/installer.json";
```

---

## Manifest Format

Example `installer.json`:

```json
{
  "version": "1.0.0",
  "release_zip_url": "https://github.com/syringeefy/xenith/releases/download/xenith/v1.0.2.zip",
  "sha256": "B70B172E681E0943781AFFCFC29CF68731A5B204AB75E6962A0722AC9A3E5C71",
  "files": [
    {"name": "libcurl.dll"},
    {$name": "VMProtectSDK64.dll"},
    {"name": "xenith.exe"},
    {"name": "zlib1.dll"}
  ],
  "prerequisites": {
    "windows_version_min": "10.0.19041",
    "vc_redist": {
      "required": true,
      "url": "https://aka.ms/vs/17/release/vc_redist.x64.exe"
    }
  }
}
```

---

## Generating the SHA-256 Hash

When updating a release, generate the SHA-256 of your ZIP and update the manifest.

PowerShell:

```powershell
Get-FileHash .\your_release.zip -Algorithm SHA256
```



Copy the resulting hash into the `sha256` field of the manifest.

---

## Run

```bash
.\target\release\bootstrapper.exe
```

Or use the included batch file:

```bash
run.bat
```

The bootstrapper lets the user choose between a standard install (AppData) or a custom directory.



*created by syringee*
