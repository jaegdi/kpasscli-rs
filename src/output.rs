use anyhow::{Result, Context};
use arboard::Clipboard;
use keepass_ng::db::{Entry, NodePtr, with_node, Node};

use std::io::Write;
use std::process::{Command, Stdio};
use crate::config::Config;

pub enum OutputType {
    Stdout,
    Clipboard,
}

impl OutputType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "stdout" => Some(OutputType::Stdout),
            "clipboard" => Some(OutputType::Clipboard),
            _ => None,
        }
    }
}

pub struct Handler {
    output_type: OutputType,
    clipboard_timeout: Option<u64>,
}

impl Handler {
    pub fn new(output_type: OutputType, clipboard_timeout: Option<u64>) -> Self {
        Self { output_type, clipboard_timeout }
    }

    pub fn output(&self, value: &str) -> Result<()> {
        match self.output_type {
            OutputType::Stdout => {
                println!("{}", value);
                Ok(())
            }
            OutputType::Clipboard => {
                #[cfg(target_os = "linux")]
                {
                    if copy_to_clipboard_linux(value).is_ok() {
                        self.spawn_background_clear()?;
                        return Ok(());
                    }
                }
                
                let mut clipboard = Clipboard::new().context("Failed to initialize clipboard")?;
                clipboard.set_text(value).context("Failed to copy to clipboard")?;
                self.spawn_background_clear()?;
                Ok(())
            }
        }
    }

    fn spawn_background_clear(&self) -> Result<()> {
        if let Some(timeout) = self.clipboard_timeout {
            if timeout > 0 {
                // Get current executable path
                let exe = std::env::current_exe()
                    .context("Failed to get current executable path")?;

                // Spawn background process to clear clipboard after timeout
                Command::new(exe)
                    .arg("--clear-clipboard-after")
                    .arg(timeout.to_string())
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .context("Failed to spawn background clipboard clearer")?;

                eprintln!("Clipboard will be cleared in {} seconds (running in background)...", timeout);
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn copy_to_clipboard_linux(value: &str) -> Result<()> {
    // Try wl-copy for Wayland
    if is_command_available("wl-copy") {
        let mut child = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()?;
            
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(value.as_bytes())?;
        }
        child.wait()?;
        return Ok(());
    }

    // Try xclip for X11
    if is_command_available("xclip") {
        let mut child = Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .stdin(Stdio::piped())
            .spawn()?;
            
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(value.as_bytes())?;
        }
        child.wait()?;
        return Ok(());
    }
    
    // Try xsel as fallback
    if is_command_available("xsel") {
         let mut child = Command::new("xsel")
            .arg("--clipboard")
            .arg("--input")
            .stdin(Stdio::piped())
            .spawn()?;
            
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(value.as_bytes())?;
        }
        child.wait()?;
        return Ok(());
    }

    Err(anyhow::anyhow!("No external clipboard tool found"))
}

#[cfg(target_os = "linux")]
fn is_command_available(program: &str) -> bool {
    Command::new("which")
        .arg(program)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn resolve_output_type(flag_out: Option<String>, cfg: &Config) -> OutputType {
    if let Some(out) = flag_out {
        if let Some(t) = OutputType::from_str(&out) {
            return t;
        }
    }
    
    if let Ok(env_out) = std::env::var("KPASSCLI_OUT") {
        if let Some(t) = OutputType::from_str(&env_out) {
            return t;
        }
    }

    if let Some(default_out) = &cfg.default_output {
        if let Some(t) = OutputType::from_str(default_out) {
            return t;
        }
    }

    OutputType::Stdout
}

pub fn show_all_fields(node: &NodePtr) {
    with_node::<Entry, _, _>(node, |entry| {
        println!("----------------------------------------");
        println!("Entry Details:");
        println!("----------------------------------------");

        if let Some(title) = entry.get_title() {
            println!("Title: {}", title);
        }
        if let Some(username) = entry.get_username() {
            println!("Username: {}", username);
        }
        if let Some(url) = entry.get_url() {
            println!("URL: {}", url);
        }
        if let Some(notes) = entry.get_notes() {
            println!("Notes: {}", notes);
        }
        
        // Custom fields?
        // We can't iterate over private fields.
        // But maybe we can print what we have.
        
        println!("----------------------------------------");
        println!("Metadata:");
        // Times
        let times = entry.get_times();
        if let Some(t) = times.get_creation() {
            println!("Created: {}", t);
        }
        if let Some(t) = times.get_last_modification() {
            println!("Modified: {}", t);
        }
        if let Some(t) = times.get_last_access() {
            println!("Accessed: {}", t);
        }
    });
}
