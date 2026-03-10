#!/bin/bash
# Helper script to copy secrets from sblite to rustdex
# Since GitHub doesn't allow reading secret values, you need to have them saved locally
# or retrieve them from your password manager/keychain

echo "=== Copy Secrets from sblite to rustdex ==="
echo ""
echo "GitHub secrets cannot be read back once set."
echo "You need to have the secret values saved locally."
echo ""

# List secrets in sblite
echo "Secrets in sblite repo:"
gh api repos/burggraf/sblite/actions/secrets --jq '.secrets[].name' 2>/dev/null || echo "Could not list secrets"
echo ""

# List secrets in rustdex (if any)
echo "Secrets currently in rustdex repo:"
gh api repos/burggraf/rustdex/actions/secrets --jq '.secrets[].name' 2>/dev/null || echo "(none or no access)"
echo ""

read -p "Do you have the secret values saved? (y/n): " confirm
if [[ $confirm != [yY] ]]; then
    echo "Please retrieve your secrets first from your password manager or keychain."
    echo ""
    echo "You'll need these 5 secrets:"
    echo "  1. APPLE_CERTIFICATE_P12 - Your Developer ID cert in base64"
    echo "  2. APPLE_CERTIFICATE_PASSWORD - The P12 file password"
    echo "  3. APPLE_ID - Your Apple ID email"
    echo "  4. APPLE_APP_PASSWORD - App-specific password from appleid.apple.com"
    echo "  5. APPLE_TEAM_ID - Your Apple Developer Team ID"
    exit 1
fi

echo ""
echo "Setting secrets in rustdex repo..."
echo ""

# Function to set a secret
set_secret() {
    local name=$1
    local value=$2
    local repo="burggraf/rustdex"

    echo "Setting $name..."
    gh secret set "$name" --repo "$repo" --body "$value"
    if [ $? -eq 0 ]; then
        echo "  ✓ $name set successfully"
    else
        echo "  ✗ Failed to set $name"
    fi
}

# Prompt for each secret
echo "--- APPLE_CERTIFICATE_P12 ---"
echo "Paste your base64-encoded P12 certificate (end with Ctrl+D):"
APPLE_CERTIFICATE_P12=$(cat)
echo ""

read -sp "Enter APPLE_CERTIFICATE_PASSWORD: " APPLE_CERTIFICATE_PASSWORD
echo ""

read -p "Enter APPLE_ID (your Apple ID email): " APPLE_ID
echo ""

read -sp "Enter APPLE_APP_PASSWORD (app-specific password): " APPLE_APP_PASSWORD
echo ""

read -p "Enter APPLE_TEAM_ID (10-character team ID): " APPLE_TEAM_ID
echo ""

echo ""
echo "Setting secrets in rustdex..."
set_secret "APPLE_CERTIFICATE_P12" "$APPLE_CERTIFICATE_P12"
set_secret "APPLE_CERTIFICATE_PASSWORD" "$APPLE_CERTIFICATE_PASSWORD"
set_secret "APPLE_ID" "$APPLE_ID"
set_secret "APPLE_APP_PASSWORD" "$APPLE_APP_PASSWORD"
set_secret "APPLE_TEAM_ID" "$APPLE_TEAM_ID"

echo ""
echo "Done! Verify with: gh api repos/burggraf/rustdex/actions/secrets"
