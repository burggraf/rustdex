# GitHub Actions Release Workflow

This workflow builds RustDex binaries for multiple platforms and creates a signed release.

## Triggering a Release

1. Go to the **Actions** tab in the GitHub repository
2. Select the **"Release RustDex Binaries"** workflow
3. Click **"Run workflow"**
4. Enter the version tag (e.g., `v0.1.0`)
5. Optionally check "Create as draft release" or "Mark as pre-release"
6. Click **"Run workflow"**

## Required Secrets

For macOS code signing and notarization, you need to configure the following secrets in your GitHub repository:

### `APPLE_CERTIFICATE_P12`
Your Developer ID Application certificate in P12 format, base64 encoded.

**To create:**
```bash
# Export your certificate from Keychain Access as a .p12 file
# Then encode it:
base64 -i Certificates.p12 | pbcopy
# Paste the output into the GitHub secret
```

### `APPLE_CERTIFICATE_PASSWORD`
The password you set when exporting the P12 certificate.

### `APPLE_ID`
Your Apple ID email address used for the Apple Developer account.

### `APPLE_APP_PASSWORD`
An app-specific password for your Apple ID.

**To create:**
1. Go to https://appleid.apple.com
2. Sign in with your Apple ID
3. Go to "App-Specific Passwords"
4. Generate a new password
5. Use this password for the secret

### `APPLE_TEAM_ID`
Your Apple Developer Team ID.

**To find:**
1. Go to https://developer.apple.com/account
2. Look for "Team ID" in the membership details

## Platforms Built

The workflow creates binaries for:

| Platform | Architecture | Filename Pattern |
|----------|--------------|------------------|
| macOS | Apple Silicon (ARM64) | `rustdex-vX.X.X-darwin-arm64.zip` |
| macOS | Intel (x64) | `rustdex-vX.X.X-darwin-amd64.zip` |
| Linux | ARM64 | `rustdex-vX.X.X-linux-arm64.zip` |
| Linux | x64 | `rustdex-vX.X.X-linux-amd64.zip` |
| Windows | ARM64 | `rustdex-vX.X.X-windows-arm64.zip` |
| Windows | x64 | `rustdex-vX.X.X-windows-amd64.zip` |

## macOS Signing Process

The workflow performs the following for macOS binaries:

1. **Import Certificate**: Creates a temporary keychain and imports the Developer ID certificate
2. **Code Sign**: Signs the binary with the Developer ID Application certificate using hardened runtime
3. **Notarize**: Submits the binary to Apple's notary service and waits for approval
4. **Cleanup**: Removes the temporary keychain

The signed and notarized binaries will run without Gatekeeper warnings on macOS.

## Troubleshooting

### Certificate not found
```
Error: No Developer ID Application identity found
```

Make sure your certificate is a "Developer ID Application" certificate (not "Mac Developer" or "Apple Development"). These are created in the Apple Developer portal under Certificates > Production.

### Notarization fails
```
Error: Unable to notarize app
```

Check that:
- `APPLE_ID` and `APPLE_APP_PASSWORD` are correct
- `APPLE_TEAM_ID` matches your developer team
- The app-specific password hasn't expired

### Build fails for specific target

Some targets may require additional dependencies. Check the build logs for missing system libraries.

## Manual Testing

After a release is created, test the binaries:

```bash
# Download and test macOS ARM binary
curl -L -o rustdex-darwin-arm64.zip https://github.com/burggraf/rustdex/releases/download/v0.1.0/rustdex-v0.1.0-darwin-arm64.zip
unzip rustdex-darwin-arm64.zip
./rustdex --version

# Verify signature (macOS)
codesign --verify --verbose rustdex
codesign -dv --verbose=4 rustdex
```
