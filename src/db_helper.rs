use anyhow::{anyhow, Context, Result};
use keepass::{Database, DatabaseKey};
use std::fs::File;

use std::path::Path;
use std::process::Command;

use crate::config::Config;

pub fn open_database(path: &str, password: &str) -> Result<Database> {
    let path = Path::new(path);
    let mut file =
        File::open(path).with_context(|| format!("Failed to open database file: {:?}", path))?;

    let key = DatabaseKey::new().with_password(password);
    Database::open(&mut file, key)
        .with_context(|| "Failed to open KeePass database. Check password or keyfile.")
}

pub fn resolve_password(
    pass_param: Option<String>,
    cfg: &Config,
    kdb_pass_env: Option<String>,
) -> Result<String> {
    if let Some(p) = pass_param {
        return resolve_password_from_source(&p);
    }
    if let Some(p) = kdb_pass_env {
        return resolve_password_from_source(&p);
    }
    if let Some(p) = &cfg.password_file {
        return resolve_password_from_source(p);
    }
    if let Some(p) = &cfg.password_executable {
        return resolve_password_from_source(p);
    }

    // Prompt user
    rpassword::prompt_password("Enter password: ").context("Failed to read password")
}

fn resolve_password_from_source(source: &str) -> Result<String> {
    let path = Path::new(source);

    // Check if it's a file (including named pipes)
    if path.exists() {
        // Check if executable
        if is_executable(path) {
            let output = Command::new(source)
                .output()
                .with_context(|| format!("Failed to execute password command: {}", source))?;

            if !output.status.success() {
                return Err(anyhow!("Password command failed"));
            }
            return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
        }

        // Read from file
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read password file: {}", source))?;
        return Ok(content.trim().to_string());
    }

    // If not a file, maybe it's a command in PATH?
    if let Ok(path) = which::which(source) {
        let output = Command::new(path)
            .output()
            .with_context(|| format!("Failed to execute password command: {}", source))?;

        if !output.status.success() {
            return Err(anyhow!("Password command failed"));
        }
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    // Treat as direct password if nothing else matches?
    // The Go code seems to treat it strictly as file or executable if passed via flag/config.
    // But if passed via env var?
    // Let's stick to the Go logic: check if file/exec, else error.
    // Wait, the Go logic says: "Password file or executable to get password".
    // It doesn't seem to support direct password string in these flags.

    Err(anyhow!(
        "Password source not found or not executable: {}",
        source
    ))
}

use std::fs;

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs::metadata(path) {
        return metadata.permissions().mode() & 0o111 != 0;
    }
    false
}

#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    // On Windows, check if the file has an executable extension
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        return ext == "exe" || ext == "bat" || ext == "cmd" || ext == "ps1";
    }
    false
}
