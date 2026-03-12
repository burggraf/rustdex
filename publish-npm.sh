#!/bin/bash

# RustDex npm Publishing Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
NPM_DIR="npm"
GITHUB_USERNAME="yourusername"  # UPDATE THIS!

echo "=== RustDex npm Publishing Script ==="
echo ""

# Get version from Cargo.toml
VERSION=$(grep "^version" Cargo.toml | head -1 | awk '{print $3}' | tr -d '"')
echo "RustDex version: $VERSION"
echo ""

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Cargo.toml not found. Please run this script from the rustdex root directory.${NC}"
    exit 1
fi

# Function to update package.json
update_package_json() {
    echo "Updating package.json version..."
    sed -i.bak "s/\"version\": \"[^\"]*\"/\"version\": \"$VERSION\"/" "$NPM_DIR/package.json"
    rm -f "$NPM_DIR/package.json.bak"
    echo -e "${GREEN}✓ Updated package.json${NC}"
}

# Function to update install.js GITHUB_REPO
update_install_js() {
    echo "Updating install.js with GitHub repository..."
    sed -i.bak "s|GITHUB_REPO = '[^']*'|GITHUB_REPO = '$GITHUB_USERNAME/rustdex'|" "$NPM_DIR/install.js"
    rm -f "$NPM_DIR/install.js.bak"
    echo -e "${GREEN}✓ Updated install.js${NC}"
}

# Function to test installation locally
test_install() {
    echo ""
    echo "=== Testing local installation ==="
    cd "$NPM_DIR"
    npm link

    echo ""
    echo -e "${GREEN}✓ Package linked locally${NC}"
    echo ""
    echo "Testing rustdex command..."
    if rustdex --version 2>/dev/null; then
        echo -e "${GREEN}✓ rustdex command works!${NC}"
    else
        echo -e "${YELLOW}⚠ rustdex command failed (this is expected if binary is not built yet)${NC}"
    fi

    echo ""
    echo "To uninstall the test installation, run:"
    echo "  npm unlink -g rustdex"
    echo ""

    cd ..
}

# Function to publish to npm
publish_to_npm() {
    echo ""
    echo "=== Publishing to npm ==="
    cd "$NPM_DIR"

    # Check if user is logged in to npm
    echo "Checking npm authentication..."
    if ! npm whoami > /dev/null 2>&1; then
        echo -e "${RED}Error: Not logged in to npm. Please run: npm login${NC}"
        exit 1
    fi

    echo -e "${GREEN}✓ Authenticated with npm${NC}"

    # Show package info before publishing
    echo ""
    echo "Package information:"
    cat package.json | grep -E '"(name|version|description)":' | head -3

    echo ""
    read -p "Ready to publish to npm? (y/N) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        npm publish
        echo -e "${GREEN}✓ Published successfully!${NC}"
        echo ""
        echo "Users can now install with: npm install -g rustdex"
    else
        echo "Publishing cancelled."
    fi

    cd ..
}

# Main menu
echo "What would you like to do?"
echo "1) Update package.json and install.js"
echo "2) Test installation locally"
echo "3) Publish to npm"
echo "4) All of the above"
echo "5) Exit"
echo ""
read -p "Select an option (1-5): " choice

case $choice in
    1)
        update_package_json
        update_install_js
        ;;
    2)
        test_install
        ;;
    3)
        publish_to_npm
        ;;
    4)
        update_package_json
        update_install_js
        test_install
        publish_to_npm
        ;;
    5)
        echo "Exiting..."
        exit 0
        ;;
    *)
        echo -e "${RED}Invalid option${NC}"
        exit 1
        ;;
esac