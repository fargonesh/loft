#!/usr/bin/env bash
# loft VSCode Extension Installer with LSP Support
# Run this from the loft repository root

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

echo "🚀 Loft VSCode Extension with LSP - Setup"
echo "==========================================="
echo ""

# Detect OS and VSCode context
if [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macOS"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="Linux"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    OS="Windows"
else
    OS="Unknown"
fi

# Detect if we're in a remote SSH session
if [ -n "$SSH_CLIENT" ] || [ -n "$SSH_TTY" ] || [ -n "$VSCODE_IPC_HOOK_CLI" ]; then
    print_status "Detected remote SSH session"
    # For remote SSH, extensions go to a different location
    if [ -n "$VSCODE_IPC_HOOK_CLI" ]; then
        # Extract connection hash from VSCode IPC hook
        CONNECTION_HASH=$(echo "$VSCODE_IPC_HOOK_CLI" | grep -oP 'vscode-ipc-[a-f0-9-]+\.sock' | sed 's/vscode-ipc-//; s/\.sock//')
        if [ -n "$CONNECTION_HASH" ]; then
            EXT_DIR="$HOME/.vscode-server/extensions/loft-0.2.0"
            print_status "Using VSCode Server extensions directory"
        else
            EXT_DIR="$HOME/.vscode-server/extensions/loft-0.2.0"
            print_warning "Could not detect connection hash, using default server path"
        fi
    else
        EXT_DIR="$HOME/.vscode-server/extensions/loft-0.2.0"
        print_status "Using VSCode Server extensions directory"
    fi
    REMOTE_MODE=true
else
    # Local installation
    EXT_DIR="$HOME/.vscode/extensions/loft-0.2.0"
    print_status "Using local VSCode extensions directory"
    REMOTE_MODE=false
fi

print_status "Detected OS: $OS"
print_status "Extension directory: $EXT_DIR"
if [ "$REMOTE_MODE" = true ]; then
    print_status "Remote SSH mode detected"
fi
echo ""

# Check if VSCode extensions directory exists
if [ "$REMOTE_MODE" = true ]; then
    # For remote SSH, check .vscode-server directory
    if [ ! -d "$HOME/.vscode-server" ]; then
        print_error "VSCode Server directory not found!"
        print_error "Please make sure you've connected to this machine via VSCode Remote SSH at least once."
        exit 1
    fi
    # Create extensions directory if it doesn't exist
    mkdir -p "$HOME/.vscode-server/extensions"
else
    # For local, check .vscode directory
    if [ ! -d "$HOME/.vscode/extensions" ]; then
        print_error "VSCode extensions directory not found!"
        print_error "Please make sure VSCode is installed."
        exit 1
    fi
fi

# Check if we're in the correct directory
if [ ! -d ".vscode-extension" ]; then
    print_error ".vscode-extension directory not found!"
    print_error "Please run this script from the loft repository root."
    exit 1
fi

# Check if Cargo.toml exists
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found!"
    print_error "Please run this script from the loft repository root."
    exit 1
fi

# Step 1: Install the LSP server
print_status "Installing loft LSP server to Cargo bin..."
if cargo install --path . --force --bin loft-lsp; then
    print_success "LSP server installed successfully"
else
    print_error "Failed to install LSP server"
    exit 1
fi

# Verify LSP binary is accessible
if command -v loft-lsp >/dev/null 2>&1; then
    LSP_PATH=$(which loft-lsp)
    print_success "LSP server installed to: $LSP_PATH"
    print_status "LSP server binary size: $(du -h "$LSP_PATH" | cut -f1)"
else
    print_error "LSP binary not found in PATH after installation"
    print_warning "Make sure ~/.cargo/bin is in your PATH"
    exit 1
fi

# Step 2: Install npm dependencies for extension
print_status "Installing npm dependencies..."
cd .vscode-extension

if command -v npm >/dev/null 2>&1; then
    if npm install; then
        print_success "npm dependencies installed"
    else
        print_error "Failed to install npm dependencies"
        exit 1
    fi
else
    print_warning "npm not found. The extension may not work properly without vscode-languageclient."
    print_warning "Please install Node.js and npm, then run: npm install"
fi

cd ..

# Step 3: Remove existing installation if present
if [ -d "$EXT_DIR" ] || [ -L "$EXT_DIR" ]; then
    print_status "Removing existing installation..."
    rm -rf "$EXT_DIR"
fi

# Check if .vscode-extension exists
if [ ! -d ".vscode-extension" ]; then
    print_error ".vscode-extension directory not found!"
    print_error "Please run this script from the loft repository root."
    exit 1
fi

# Step 4: Install the extension
print_status "Installing extension..."
cp -r .vscode-extension "$EXT_DIR"

# Step 5: Verify LSP server accessibility
print_status "Verifying LSP server accessibility..."

# Check if loft-lsp is in PATH (should be after cargo install)
if command -v loft-lsp >/dev/null 2>&1; then
    print_success "loft-lsp available in PATH: $(which loft-lsp)"
else
    print_error "loft-lsp not found in PATH"
    print_warning "Make sure ~/.cargo/bin is in your PATH environment variable"
    print_warning "Add this to your shell profile: export PATH=\"\$HOME/.cargo/bin:\$PATH\""
fi

# Step 6: Test LSP server
print_status "Testing LSP server..."
if timeout 5s loft-lsp --help >/dev/null 2>&1; then
    print_success "LSP server responds correctly"
else
    print_warning "LSP server test failed or timed out"
fi

if [ $? -eq 0 ]; then
    echo ""
    print_success "🎉 loft VSCode Extension with LSP installed successfully!"
    echo ""
    echo "📋 Installation Summary:"
    echo "   Extension: $EXT_DIR"
    echo "   LSP Server: $(which loft-lsp 2>/dev/null || echo 'Not found in PATH')"
    echo "   Version: 0.2.0"
    echo ""
    echo "🔧 Features Available:"
    echo "   ✅ Syntax highlighting"
    echo "   ✅ Auto-completion (keywords)"
    echo "   ✅ Hover information"
    echo "   ✅ Document synchronization"
    echo ""
    echo "🚀 Next Steps:"
    if [ "$REMOTE_MODE" = true ]; then
        echo "   1. Reload the VSCode window (Ctrl+Shift+P -> 'Developer: Reload Window')"
        echo "   2. Open any .lf file in the remote workspace"
        echo "   3. Check 'loft LSP' in Output panel for server logs"
        echo "   4. Test auto-completion with Ctrl+Space"
        echo ""
        echo "   Note: For remote SSH, you may need to reload the window instead of restarting VSCode"
    else
        echo "   1. Restart VSCode (close all windows and reopen)"
        echo "   2. Open any .lf file"
        echo "   3. Check 'loft LSP' in Output panel for server logs"
        echo "   4. Test auto-completion with Ctrl+Space"
    fi
    echo ""
    echo "📂 Test files available:"
    echo "   - .vscode-extension/samples/sample.lf (if exists)"
    echo "   - examples/*.lf"
    echo ""
    echo "📖 Documentation:"
    echo "   - LSP Guide: .vscode-extension/LSP_GUIDE.md"
    echo "   - Quick Start: .vscode-extension/QUICKSTART.md (if exists)"
    echo "   - Full Guide: .vscode-extension/README.md"
    echo ""
    print_status "For LSP troubleshooting, see LSP_GUIDE.md"
    echo ""
    
    # Optional: Offer to restart/reload VSCode
    if [ "$REMOTE_MODE" = true ]; then
        echo ""
        print_status "For remote SSH connections, you may need to reload the VSCode window."
        print_status "Use Ctrl+Shift+P -> 'Developer: Reload Window' or restart the SSH connection."
    else
        echo ""
        read -p "Would you like to restart VSCode now? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_status "Restarting VSCode..."
            if command -v code >/dev/null 2>&1; then
                # Kill existing VSCode instances
                pkill -f "Visual Studio Code" 2>/dev/null || true
                sleep 2
                # Start VSCode
                code . 2>/dev/null &
                print_success "VSCode restarted"
            else
                print_warning "VSCode command 'code' not found. Please restart VSCode manually."
            fi
        fi
    fi
    
    print_success "Installation complete! 🚀"
else
    print_error "Installation failed!"
    exit 1
fi
