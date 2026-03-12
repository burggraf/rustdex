# RustDex npm Package Setup Guide

This guide explains how to set up and publish the RustDex npm package so users can install it with `npm install -g rustdex`.

## How It Works

The npm package is a lightweight wrapper that:
1. Detects the user's platform and architecture during installation
2. Downloads the appropriate binary from GitHub Releases
3. Installs it to the npm global bin directory

The `.zip` format from your GitHub releases is perfect - we just download and extract it!

## Files Structure

```
rustdex/
├── npm/
│   ├── package.json          # npm package configuration
│   ├── install.js            # Downloads and extracts the binary
│   ├── uninstall.js          # Removes the binary on uninstall
│   ├── index.js              # Stub file
│   ├── bin/
│   │   └── rustdex          # Wrapper script
│   ├── README.md            # npm-specific documentation
│   └── .npmignore           # Files to exclude from npm package
└── publish-npm.sh           # Helper script for publishing
```

## Platform Support

The package automatically handles these platforms:
- **macOS**: arm64 (Apple Silicon), x64 (Intel)
- **Linux**: arm64, amd64
- **Windows**: arm64, amd64

## Before Publishing

### 1. Update Configuration

Edit `npm/install.js` and change the GitHub repository:

```javascript
// Line ~11 - Change this to your GitHub username
const GITHUB_REPO = 'yourusername/rustdex';
```

Edit `npm/package.json` and update:
- `repository.url`: Your GitHub repository URL
- `author`: Your name and email
- `version`: Should match the RustDex version in `Cargo.toml`

### 2. Create a GitHub Release

Use the GitHub Action workflow to create a release with binaries:

```bash
# Go to GitHub Actions → Release RustDex Binaries → Run workflow
# Enter version (e.g., v0.4.0)
```

Or manually trigger via the API/UI.

### 3. Verify Release Assets

Make sure these files exist in your GitHub release:
- `rustdex-v0.4.0-darwin-arm64.zip`
- `rustdex-v0.4.0-darwin-x64.zip`
- `rustdex-v0.4.0-linux-arm64.zip`
- `rustdex-v0.4.0-linux-amd64.zip`
- `rustdex-v0.4.0-windows-arm64.zip`
- `rustdex-v0.4.0-windows-amd64.zip`

## Publishing to npm

### Option 1: Use the Helper Script

```bash
chmod +x publish-npm.sh
./publish-npm.sh
```

Follow the prompts to:
1. Update `package.json` and `install.js`
2. Test installation locally
3. Publish to npm

### Option 2: Manual Publishing

```bash
# Navigate to npm directory
cd npm

# Verify package.json
cat package.json | grep version

# Test locally (optional)
npm link
rustdex --version
npm unlink -g rustdex

# Login to npm (first time only)
npm login

# Publish
npm publish
```

## Testing the Installation

Before publishing, test locally:

```bash
cd npm
npm link
rustdex --version
rustdex --help
rustdex list-repos
npm unlink -g rustdex
```

After publishing, test from a clean environment:

```bash
npm install -g rustdex
rustdex --version
```

## Version Management

When you release a new version of RustDex:

1. Update `Cargo.toml` version
2. Create GitHub release with new binaries
3. Update `npm/package.json` version to match
4. Update `npm/install.js` version if needed
5. Publish to npm: `npm publish`

The `npm/package.json` version **must match** the GitHub release tag.

## Troubleshooting

### "Release not found (404)"
- Check that the GitHub release exists: `https://github.com/yourusername/rustdex/releases`
- Verify the version number matches exactly
- Check that the zip files follow the naming convention

### "Unsupported platform"
- The user's platform/architecture isn't supported by the current build
- Check GitHub Releases for available binaries
- Consider building for additional platforms

### "Failed to extract zip"
- **Unix**: Install unzip: `sudo apt-get install unzip` or `brew install unzip`
- **Windows**: The installer uses PowerShell; if that fails, it will try to use `adm-zip`

### Binary not executable on Unix
- The install script runs `chmod +x` automatically
- If it fails, manually run: `chmod +x $(npm root -g)/rustdex/bin/rustdex`

### npm install hangs
- Check internet connection
- Try with `--verbose` flag: `npm install -g rustdex --verbose`
- Check GitHub status page for rate limits

## Integration with Development Workflow

### Automated Publishing

You can add a step to your release workflow that automatically publishes to npm after creating the GitHub release:

```yaml
# Add to .github/workflows/release.yml
publish-npm:
  name: Publish to npm
  needs: release
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: '18'
        registry-url: 'https://registry.npmjs.org'
    - name: Publish to npm
      run: |
        cd npm
        npm publish
      env:
        NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

You'll need to:
1. Create an npm access token: https://www.npmjs.com/settings/tokens
2. Add it as `NPM_TOKEN` secret in your GitHub repository settings

## Next Steps

1. ✅ Files are created in `npm/` directory
2. ⬜ Update `GITHUB_REPO` in `npm/install.js` with your GitHub username
3. ⬜ Update `package.json` with your author info and repository URL
4. ⬜ Create a GitHub release with binaries
5. ⬜ Test the installation locally
6. ⬜ Publish to npm: `npm publish`

## User Experience After Setup

Once published, users can simply run:

```bash
npm install -g rustdex
```

And they'll have a working RustDex binary on their PATH, automatically selected for their platform!