# kpasscli

A secure command-line interface for KeePass database entries designed for automation, security, and seamless integration with workflows.

**Built with Rust** for performance, security, and cross-platform compatibility.

kpasscli provides a secure way to query KeePass database entries without exposing passwords in scripts or logs. It's ideal for developers, system administrators, and security-conscious users who need to programmatically access credentials while maintaining strict security standards.

## Features

- 🔒 **Security-first design**: Passwords never appear in command line history or process lists
- 🔄 **Flexible search**: Supports absolute paths, relative paths, and simple names
- 🧠 **Smart field selection**: Default to password field or customize with `--field-name`
- 📦 **Output control**: Print to stdout or copy to clipboard
- ⚙️ **Configurable**: Customizable via environment variables or config files
- 🛡️ **Secure password handling**: Supports password files and secure executables
- ⏱️ **Background clipboard clearing**: Automatically clears clipboard after configurable timeout
- 🚀 **Fast**: Built with Rust for optimal performance

## Installation

### From Source

```bash
cargo build --release
```

The binary will be available at `target/release/kpasscli`.

### Cross-Platform Builds

Use the provided build script to create binaries for multiple platforms:

```bash
./build.sh
```

This creates binaries for:
- Linux x86_64
- Windows x86_64
- macOS x86_64 (Intel)
- macOS ARM64 (Apple Silicon)

**Prerequisites for cross-compilation:**
```bash
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

## Usage

### Synopsis

```
kpasscli [OPTIONS]
```

### Options

The CLI flags below reflect the current implementation in `src/args.rs`.

| Option | Env Var | Config Key | Description |
|--------|---------|------------|-------------|
| `-p, --kdbpath <PATH>` | `KPASSCLI_KDBPATH` | `database_path` | Path to KeePass database file |
| `-w, --kdbpassword <PATH>` | `KPASSCLI_KDBPASSWORD` | `password_file` or `password_executable` | Password source: file path or executable |
| `-i, --item <NAME>` | — | — | Entry to search for (required) |
| `-f, --fieldname <FIELD>` | — | — | Field to retrieve (default: `Password`) |
| `-o, --out <stdout\|clipboard>` | `KPASSCLI_OUT` | `default_output` | Output destination |
| `-C, --Clip` | — | — | Shortcut: force clipboard output (overrides env/config) |
| `-c, --case-sensitive` | — | — | Enable case‑sensitive search |
| `-e, --exact-match` | — | — | Enable exact match search |
| `--show-all` | — | — | Print all common and custom fields for the entry |
| `-t, --totp` | — | — | Output TOTP token generated from the entry’s `otp` field |
| `-T, --password-totp` (alias `--pt`) | — | — | Output `<password> <totp>` on one line |
| `--create-config` (alias `--cc`) | — | — | Create example config file in current directory |
| `--print-config` (alias `--pc`) | — | — | Print the effective configuration and file path |
| `--config <PATH>` | — | — | Path to config file (default: `~/.config/kpasscli/config.yaml`) |
| `-d, --debug` | — | — | Print timing/debug info to stderr |
| `-v, --verify` | — | — | Enable verify messages (informational) |
| `-h, --help` | — | — | Print help |

## Search Behavior

### Absolute Path
```bash
kpasscli -p db.kdbx -w pass.txt -i /Root/Personal/Banking/Account
```
Searches for an exact match at the specified location in the database.

### Relative Path
```bash
kpasscli -p db.kdbx -w pass.txt -i Banking/Account
```
Searches through all groups for a matching subpath. Returns error if multiple matches found.

### Simple Name
```bash
kpasscli -p db.kdbx -w pass.txt -i Account
```
Searches all matching entries regardless of location. Returns error if multiple matches found.

## Configuration

kpasscli uses a layered configuration approach:
1. Command-line flags (highest priority)
2. Environment variables
3. Config file (`~/.config/kpasscli/config.yaml`)

### Configuration File Format

```yaml
database_path: /path/to/your/database.kdbx
# stdout | clipboard
default_output: stdout

# Choose ONE of the following password sources
password_file: /path/to/your/password.txt
# or
password_executable: /path/to/your/password_executable.sh

# seconds; 0 disables background clearing
clipboard_timeout: 15
```

Create an example config file:
```bash
kpasscli --create-config
```

### Password Retrieval Methods

**⚠️ Security Note**: Protect password files and executables with appropriate file permissions.

1. **Password File**: Plain text file containing the database password
   ```yaml
   password_file: /path/to/password.txt
   ```

2. **Password Executable**: Script or program that outputs the password
   ```yaml
   password_executable: /path/to/get_password.sh
   ```

3. **Environment Variable**: Set `KPASSCLI_KDBPASSWORD` with file path or executable

### Environment Variables

- `KPASSCLI_KDBPATH` — path to the KeePass database file
- `KPASSCLI_KDBPASSWORD` — path to password file or password‑producing executable
- `KPASSCLI_OUT` — `stdout` or `clipboard`

Precedence (highest first): command‑line flags → environment variables → config file.

## Examples

### Basic Usage
```bash
# Get password for specific entry
kpasscli -p db.kdbx -w pass.txt -i "/Personal/Email/Gmail"

# Get username instead of password
kpasscli -p db.kdbx -w pass.txt -i "Gmail" -f UserName

# Copy password to clipboard using -o
kpasscli -p db.kdbx -w pass.txt -i "Gmail" -o clipboard

# Or use the shortcut flag to force clipboard output
kpasscli -p db.kdbx -w pass.txt -i "Gmail" -C
```

### Using Config File
```bash
# With config file at default location (~/.config/kpasscli/config.yaml)
kpasscli -i "Gmail"

# With custom config file
kpasscli -c my_config.yaml -i "Gmail"
```

### Advanced Search
```bash
# Case-sensitive search
kpasscli -p db.kdbx -w pass.txt -i "Account" --case-sensitive

# Exact match only
kpasscli -p db.kdbx -w pass.txt -i "MyAccount" --exact-match

# Show all fields of an entry
kpasscli -p db.kdbx -w pass.txt -i "Gmail" --show-all

### TOTP

```bash
# Output only the TOTP token (entry must contain an `otp` field with an otpauth URL)
kpasscli -p db.kdbx -w pass.txt -i "Gmail" --totp

# Output password and TOTP token in one line
kpasscli -p db.kdbx -w pass.txt -i "Gmail" --password-totp
```

### Using Environment Variables
```bash
export KPASSCLI_KDBPATH=/path/to/db.kdbx
export KPASSCLI_KDBPASSWORD=/path/to/pass.txt
export KPASSCLI_OUT=clipboard

kpasscli -i "Gmail"
```

## Clipboard Timeout

When using clipboard output, kpasscli can automatically clear the clipboard after a configurable timeout. This happens in a background process, so the command returns immediately.

Configure timeout in `config.yaml`:
```yaml
clipboard_timeout: 15  # Clear clipboard after 15 seconds
```

The clipboard clearing runs in the background, allowing the command to return to the shell prompt immediately while the cleanup happens asynchronously.

Notes for Linux clipboard support:
- kpasscli first tries Wayland’s `wl-copy`, then X11’s `xclip`, then `xsel`.
- If none are available, it falls back to a cross‑platform clipboard library where possible.

## Security Considerations

- ✅ Passwords are **never** exposed in command line arguments
- ✅ Database passwords must be provided via file or executable (never directly)
- ✅ Clipboard contents are automatically cleared after configurable delay
- ✅ Background processes handle cleanup without blocking main application
- ⚠️ Be cautious when using clipboard output on shared systems
- ⚠️ Protect password files with appropriate permissions (chmod 600)
- ⚠️ Store config files in secure locations with restricted access

## Building from Source

### Prerequisites
- Rust 1.70 or later
- Cargo

### Build Release Binary
```bash
cargo build --release
```

### Run Tests
```bash
cargo test
```

### Install Locally
```bash
cargo install --path .
```

## Cross-Compilation

For cross-platform builds, additional toolchains may be required:
- **Windows**: mingw-w64 (`apt install mingw-w64`)
- **macOS** (from Linux): osxcross or similar

See `build.sh` for automated cross-compilation setup.

## License

GNU General Public License Version 3, 29 June 2007

See [LICENSE](LICENSE) file for full details.

## Author

Dirk Jäger

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
