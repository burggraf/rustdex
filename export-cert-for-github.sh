#!/bin/bash
# Export Developer ID certificate from macOS Keychain and prepare for GitHub Actions

set -e

echo "=== Export Developer ID Certificate for GitHub Actions ==="
echo ""

# Certificate details
TEAM_ID="HEGN9W2S9J"
CERT_NAME="Developer ID Application: Mantis Bible Company ($TEAM_ID)"

echo "Found certificate: $CERT_NAME"
echo ""

# Create temp directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

echo "Step 1: Exporting certificate from Keychain..."
echo "You'll need to enter your macOS login password and create a password for the P12 file."
echo ""

# Export the certificate
security export -k login.keychain-db \
    -t identities \
    -f pkcs12 \
    -o "$TEMP_DIR/rustdex-cert.p12" \
    -P "" \
    2>/dev/null || {
    echo "Note: If prompted, enter your macOS password to allow keychain access"
    security export -k login.keychain-db \
        -t identities \
        -f pkcs12 \
        -o "$TEMP_DIR/rustdex-cert.p12"
}

echo ""
echo "Step 2: Verifying exported certificate..."

# Check if export worked
if [ ! -f "$TEMP_DIR/rustdex-cert.p12" ]; then
    echo "ERROR: Failed to export certificate"
    exit 1
fi

# Check file size
FILE_SIZE=$(stat -f%z "$TEMP_DIR/rustdex-cert.p12")
if [ $FILE_SIZE -lt 100 ]; then
    echo "ERROR: Exported file is too small ($FILE_SIZE bytes). Something went wrong."
    exit 1
fi

echo "✓ Certificate exported successfully (${FILE_SIZE} bytes)"
echo ""

echo "Step 3: Converting to base64 for GitHub..."

# Encode to base64
BASE64_CERT=$(base64 -i "$TEMP_DIR/rustdex-cert.p12")

echo ""
echo "=== CERTIFICATE READY ==="
echo ""
echo "The base64-encoded certificate is below (first 100 chars):"
echo "${BASE64_CERT:0:100}..."
echo ""
echo "Total length: ${#BASE64_CERT} characters"
echo ""

# Save to file for easy copying
CERT_FILE="$TEMP_DIR/rustdex-cert-base64.txt"
echo "$BASE64_CERT" > "$CERT_FILE"

echo "Full certificate saved to: $CERT_FILE"
echo ""

echo "=== NEXT STEPS ==="
echo ""
echo "1. Copy the base64 certificate to GitHub secret APPLE_CERTIFICATE_P12:"
echo ""
echo "   pbcopy < $CERT_FILE"
echo "   OR"
echo "   cat $CERT_FILE | pbcopy"
echo ""
echo "2. Then paste it into GitHub:"
echo "   https://github.com/burggraf/rustdex/settings/secrets/actions"
echo ""
echo "3. Also set these secrets:"
echo "   - APPLE_CERTIFICATE_PASSWORD: [the password you used when exporting]"
echo "   - APPLE_ID: markb@mantisbible.com"
echo "   - APPLE_APP_PASSWORD: [generate at https://appleid.apple.com]"
echo "   - APPLE_TEAM_ID: $TEAM_ID"
echo ""

# Open the file for easy copying
if command -v pbcopy &> /dev/null; then
    echo "Copying to clipboard now..."
    cat "$CERT_FILE" | pbcopy
    echo "✓ Certificate copied to clipboard!"
    echo ""
    echo "PASTE THIS VALUE into GitHub secret 'APPLE_CERTIFICATE_P12':"
    echo "(Clipboard contains the full base64 string)"
fi

echo ""
echo "Press Enter to finish and clean up temp files..."
read
