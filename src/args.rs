use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to KeePass database file
    #[arg(short = 'p', long = "kdbpath", alias = "p")]
    pub kdb_path: Option<String>,

    /// Password file or executable to get password
    #[arg(short = 'w', long = "kdbpassword", alias = "w")]
    pub kdb_password: Option<String>,

    /// Item to search for
    #[arg(short = 'i', long = "item", alias = "i")]
    pub item: Option<String>,

    /// Field name to retrieve
    #[arg(short = 'f', long = "fieldname", default_value = "Password")]
    pub field_name: String,

    /// Output type (clipboard/stdout)
    #[arg(short = 'o', long = "out")]
    pub out: Option<String>,

    /// Enable exact match search
    #[arg(short = 'C', long = "Clip")]
    pub clipboard: bool,

    /// Enable case-sensitive search
    #[arg(short = 'c', long = "case-sensitive", alias = "cs")]
    pub case_sensitive: bool,

    /// Enable exact match search
    #[arg(short = 'e', long = "exact-match")]
    pub exact_match: bool,

    /// Show manual page
    #[arg(short = 'm', long = "man")]
    pub show_man: bool,

    /// Enable debug logging
    #[arg(short = 'd', long = "debug")]
    pub debug: bool,

    /// Enable verify messages
    #[arg(short = 'v', long = "verify")]
    pub verify: bool,

    /// Create an example config file
    #[arg(long = "create-config", alias = "cc")]
    pub create_config: bool,

    /// Print current configuration
    #[arg(long = "print-config", alias = "pc")]
    pub print_config: bool,

    /// Path to configuration file
    #[arg(long = "config", default_value = "~/.config/kpasscli/config.yaml")]
    pub config_path: String,

    /// Show all fields
    #[arg(long, help = "Show all fields of an entry")]
    pub show_all: bool,

    /// Hidden argument for background clipboard clearing (internal use only)
    #[arg(long, hide = true)]
    pub clear_clipboard_after: Option<u64>,

    /// Get TOTP token
    #[arg(short = 't', long = "totp")]
    pub totp: bool,

    /// Get password and TOTP token
    #[arg(short = 'T', long = "password-totp", alias = "pt")]
    pub password_totp: bool,
}
