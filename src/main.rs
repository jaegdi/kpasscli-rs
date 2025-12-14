mod args;
mod config;
mod db_helper;
mod otp;
mod output;
mod search;

use anyhow::{anyhow, Result};
use clap::Parser;
use keepass::db::Entry;
use std::process;

use crate::args::Args;
use crate::config::Config;
use crate::db_helper::{open_database, resolve_password};
use crate::output::{resolve_output_type, show_all_fields, Handler};
use crate::search::{Finder, SearchOptions};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    // Handle background clipboard clearing
    if let Some(seconds) = args.clear_clipboard_after {
        std::thread::sleep(std::time::Duration::from_secs(seconds));
        clear_clipboard()?;
        return Ok(());
    }

    if args.create_config {
        Config::create_example("config.yaml")?;
        println!("Example config file 'config.yaml' created successfully.");
        return Ok(());
    }

    let config = Config::load(&args.config_path)?;

    if args.print_config {
        println!("Current used Configuration: {}", config.config_file_path);
        println!("------------------------------------------");
        println!("Database Path: {:?}", config.database_path);
        println!("Default Output: {:?}", config.default_output);
        println!("Password File: {:?}", config.password_file);
        println!("Password Executable: {:?}", config.password_executable);
        println!("Clipboard Timeout: {:?}", config.clipboard_timeout);
        println!("------------------------------------------");
        return Ok(());
    }

    let item = args
        .item
        .ok_or_else(|| anyhow!("item parameter is required"))?;

    let db_path = args
        .kdb_path
        .or_else(|| std::env::var("KPASSCLI_KDBPATH").ok())
        .or(config.database_path.clone())
        .ok_or_else(|| anyhow!("no KeePass database path provided"))?;

    let kdb_pass_env = std::env::var("KPASSCLI_kdbpassword").ok();
    let password = resolve_password(args.kdb_password, &config, kdb_pass_env)?;

    let start = std::time::Instant::now();
    let db = open_database(&db_path, &password)?;
    if args.debug {
        eprintln!("Database opened in: {:?}", start.elapsed());
    }

    let finder = Finder::new(
        &db,
        SearchOptions {
            case_sensitive: args.case_sensitive,
            exact_match: args.exact_match,
        },
    );

    let results = finder.find(&item)?;

    if results.is_empty() {
        return Err(anyhow!("no items found"));
    }

    if results.len() > 1 {
        for result in &results {
            eprintln!("- {}", result.path);
        }
        return Err(anyhow!("multiple items found"));
    }

    let result = &results[0];

    if args.show_all {
        show_all_fields(&result.entry);
        return Ok(());
    }

    let mut value = get_field_value(&result.entry, &args.field_name)?;

    if args.totp || args.password_totp {
        let totp_url = get_field_value(&result.entry, "otp")
            .map_err(|_| anyhow!("Entry has no TOTP configuration"))?;

        let token = otp::generate_totp(&totp_url)?;

        if args.totp {
            value = token;
        } else if args.password_totp {
            let password = get_field_value(&result.entry, "Password")?;
            value = format!("{}{}", password, token);
        }
    }

    let output_type = resolve_output_type(args.out, args.clipboard, &config);
    let handler = Handler::new(output_type, config.clipboard_timeout);
    handler.output(&value)?;

    Ok(())
}

fn clear_clipboard() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use std::io::Write;
        use std::process::{Command, Stdio};

        // Try wl-copy
        if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(b"");
            }
            let _ = child.wait();
            return Ok(());
        }

        // Try xclip
        if let Ok(mut child) = Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(b"");
            }
            let _ = child.wait();
            return Ok(());
        }

        // Try xsel
        if let Ok(mut child) = Command::new("xsel")
            .arg("--clipboard")
            .arg("--input")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(b"");
            }
            let _ = child.wait();
            return Ok(());
        }
    }

    // Fallback to arboard
    use arboard::Clipboard;
    let mut clipboard = Clipboard::new()?;
    clipboard.set_text("")?;
    Ok(())
}

fn get_field_value(entry: &Entry, field_name: &str) -> Result<String> {
    if field_name.eq_ignore_ascii_case("Title") {
        return Ok(entry.get_title().unwrap_or_default().to_string());
    }
    if field_name.eq_ignore_ascii_case("UserName") {
        return Ok(entry.get_username().unwrap_or_default().to_string());
    }
    if field_name.eq_ignore_ascii_case("Password") {
        return Ok(entry.get_password().unwrap_or_default().to_string());
    }
    if field_name.eq_ignore_ascii_case("URL") {
        return Ok(entry.get_url().unwrap_or_default().to_string());
    }
    if field_name.eq_ignore_ascii_case("Notes") {
        return Ok(entry.get("Notes").unwrap_or_default().to_string());
    }
    // Custom fields
    if let Some(val) = entry.get(field_name) {
        return Ok(val.to_string());
    }

    // Also check case insensitive for standard fields if not found above?
    // Or maybe `fields` keys are case sensitive?

    Err(anyhow!("Field '{}' not found", field_name))
}
