# Installation

## Using the Install Script (Recommended)

The easiest way to install loft is using the official install script:

```bash
curl -fsSL https://loft.fargone.sh/install.sh | sh
```

This script will:
- Detect your operating system and architecture
- Download the appropriate binary
- Install it to /usr/local/bin or ~/.local/bin
- Make it executable

After installation, verify it works:

```bash
loft --version
```

## Manual Installation

### From GitHub Releases

1. Visit the [releases page](https://github.com/tascord/twang/releases)
2. Download the binary for your platform:
   - `loft-linux` for Linux
   - `loft-macos` for macOS
   - `loft-windows.exe` for Windows
3. Make it executable (Unix-like systems):
   ```bash
   chmod +x loft-linux
   ```
4. Move it to a directory in your PATH:
   ```bash
   sudo mv loft-linux /usr/local/bin/loft
   ```

### Building from Source

Requirements:
- Rust toolchain (1.70 or later)
- Git

Steps:

```bash
# Clone the repository
git clone https://github.com/tascord/twang.git
cd twang

# Build the release binary
cargo build --release

# The binary will be at target/release/loft
# Copy it to your PATH
sudo cp target/release/loft /usr/local/bin/
```

## VSCode Extension

For the best development experience, install the loft VSCode extension:

1. Open VSCode
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "loft"
4. Click Install

The extension provides:
- Syntax highlighting
- Code completion
- Error diagnostics
- Go to definition
- Hover information
- Code formatting

## Next Steps

Now that loft is installed, let's write your first program: [Hello World](./hello-world.md)
