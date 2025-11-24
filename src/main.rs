mod args;
mod config;
mod keepass;
mod search;
mod output;

use anyhow::{Result, anyhow};
use clap::Parser;
use std::process;
use keepass_ng::db::{Entry, NodePtr, with_node, Node};

use crate::args::Args;
use crate::config::Config;
use crate::keepass::{open_database, resolve_password};
use crate::search::{Finder, SearchOptions};
use crate::output::{Handler, resolve_output_type, show_all_fields};

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

    let item = args.item.ok_or_else(|| anyhow!("item parameter is required"))?;

    let db_path = args.kdb_path
        .or_else(|| std::env::var("KPASSCLI_KDBPATH").ok())
        .or(config.database_path.clone())
        .ok_or_else(|| anyhow!("no KeePass database path provided"))?;

    let kdb_pass_env = std::env::var("KPASSCLI_kdbpassword").ok();
    let password = resolve_password(args.kdb_password, &config, kdb_pass_env)?;

    let db = open_database(&db_path, &password)?;

    let finder = Finder::new(&db, SearchOptions {
        case_sensitive: args.case_sensitive,
        exact_match: args.exact_match,
    });

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
        show_all_fields(&result.node);
        return Ok(());
    }

    let value = get_field_value(&result.node, &args.field_name)?;

    let output_type = resolve_output_type(args.out, &config);
    let handler = Handler::new(output_type, config.clipboard_timeout);
    handler.output(&value)?;

    Ok(())
}

fn clear_clipboard() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use std::process::{Command, Stdio};
        use std::io::Write;

        // Try wl-copy
        if let Ok(mut child) = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
        {
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

fn get_field_value(node: &NodePtr, field_name: &str) -> Result<String> {
    // Try to treat as Entry
    if let Some(val) = with_node::<Entry, _, _>(node, |entry| {
        if field_name.eq_ignore_ascii_case("Title") {
            return Some(entry.get_title().unwrap_or_default().to_string());
        }
        if field_name.eq_ignore_ascii_case("UserName") {
            return Some(entry.get_username().unwrap_or_default().to_string());
        }
        if field_name.eq_ignore_ascii_case("Password") {
            return Some(entry.get_password().unwrap_or_default().to_string());
        }
        if field_name.eq_ignore_ascii_case("URL") {
            return Some(entry.get_url().unwrap_or_default().to_string());
        }
        if field_name.eq_ignore_ascii_case("Notes") {
            return Some(entry.get_notes().unwrap_or_default().to_string());
        }
        // Custom fields
        if let Some(val) = entry.get(field_name) {
            return Some(val.to_string());
        }
        None
    }) {
        if let Some(v) = val {
            return Ok(v);
        }
    }
    
    // If it's a group, maybe we can get title/notes?
    // But usually we search for entries.
    
    Err(anyhow!("Field '{}' not found", field_name))
}
