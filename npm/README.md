# RustDex npm package

This directory contains the npm package for RustDex, which allows users to install the RustDex binary via npm:

```bash
npm install -g rustdex
```

## How it works

The npm package acts as a wrapper that:
1. Detects the user's platform and architecture
2. Downloads the appropriate binary from GitHub Releases
3. Installs it to the npm global bin directory

## Publishing

To publish to npm:

1. Update `GITHUB_REPO` in `install.js` with your GitHub username
2. Update author and repository URLs in `package.json`
3. Ensure you have an npm account: `npm adduser`
4. Publish: `npm publish`

### Publishing Checklist

- [ ] Update version in `package.json` to match RustDex release
- [ ] Create GitHub release with binaries (use the workflow)
- [ ] Update `GITHUB_REPO` in `install.js`
- [ ] Test installation on each platform
- [ ] Run `npm publish`

## Version Management

The npm package version should match the RustDex release version. When you release a new version of RustDex:

1. Update `package.json` version
2. Create GitHub release with binaries
3. Publish to npm: `npm publish`

## Platform Support

Currently supports:
- macOS: arm64 (Apple Silicon), x64 (Intel)
- Linux: arm64, amd64
- Windows: arm64, amd64

## Troubleshooting

### Installation fails

1. Check if the release exists on GitHub
2. Verify your platform is supported
3. Ensure you have network access
4. On Windows: Ensure you have PowerShell or install adm-zip
5. On Unix: Ensure you have `unzip` installed

### Binary not found

The binary is installed to:
- npm global bin directory (typically `/usr/local/bin` on macOS/Linux)
- Check with: `npm config get prefix`

## Testing locally

Before publishing, you can test the installation:

```bash
cd npm
npm link
rustdex --version
npm unlink -g rustdex
```

## Dependencies

- `node-fetch`: For downloading binaries (optional, using native modules)
- `adm-zip`: For extracting on Windows (will be added if needed)

Note: The current implementation uses native `unzip` on Unix and PowerShell on Windows to avoid heavy dependencies. For better Windows support, consider adding `adm-zip` as a dependency.