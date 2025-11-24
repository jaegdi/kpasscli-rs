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

| Option | Environment Variable | Config File | Description |
|--------|---------------------|-------------|-------------|
| `-p, --kdb-path <PATH>` | `KPASSCLI_KDBPATH` | `database_path` | Path to KeePass database file |
| `-w, --kdb-password <PATH>` | `KPASSCLI_kdbpassword` | `password_file` or `password_executable` | Path to password file or executable |
| `-i, --item <NAME>` | - | - | Entry to search for (required) |
| `-f, --field-name <FIELD>` | - | - | Field to retrieve (default: Password) |
| `-o, --out <TYPE>` | `KPASSCLI_OUT` | `default_output` | Output type: stdout or clipboard |
| `-c, --config <PATH>` | - | - | Path to config file (default: `~/.config/kpasscli/config.yaml`) |
| `--case-sensitive` | - | - | Enable case-sensitive search |
| `--exact-match` | - | - | Enable exact match search |
| `--show-all` | - | - | Show all fields of an entry |
| `--create-config` | - | - | Create example config file |
| `--print-config` | - | - | Print current configuration |
| `--verify` | - | - | Verify database can be opened |
| `-h, --help` | - | - | Print help |

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
database_path: /path/to/database.kdbx
default_output: clipboard
password_file: /path/to/password.txt
clipboard_timeout: 15  # seconds, 0 to disable
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

3. **Environment Variable**: Set `KPASSCLI_kdbpassword` with file path or executable

## Examples

### Basic Usage
```bash
# Get password for specific entry
kpasscli -p db.kdbx -w pass.txt -i "/Personal/Email/Gmail"

# Get username instead of password
kpasscli -p db.kdbx -w pass.txt -i "Gmail" -f UserName

# Copy password to clipboard (clears after 15 seconds if configured)
kpasscli -p db.kdbx -w pass.txt -i "Gmail" -o clipboard
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
```

### Using Environment Variables
```bash
export KPASSCLI_KDBPATH=/path/to/db.kdbx
export KPASSCLI_kdbpassword=/path/to/pass.txt
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
