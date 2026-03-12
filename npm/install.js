#!/usr/bin/env node

/**
 * RustDex Binary Installer
 * Downloads the appropriate binary from GitHub Releases and installs it
 */

const https = require('https');
const http = require('http');
const fs = require('fs');
const path = require('path');
const os = require('os');
const { execSync } = require('child_process');

const GITHUB_REPO = 'burggraf/rustdex';
const VERSION = require('./package.json').version;

// Platform detection mapping - matches GitHub release naming
const PLATFORM_MAP = {
  'darwin': {
    'arm64': 'darwin-arm64',
    'x64': 'darwin-x64'
  },
  'linux': {
    'arm64': 'linux-arm64',
    'x64': 'linux-amd64'
  },
  'win32': {
    'arm64': 'windows-arm64',
    'x64': 'windows-amd64'
  }
};

function getPlatformInfo() {
  const platform = os.platform();
  const arch = os.arch();

  if (!PLATFORM_MAP[platform]) {
    console.error(`Unsupported platform: ${platform}`);
    console.error('Supported platforms: macOS, Linux, Windows');
    throw new Error(`Unsupported platform: ${platform}`);
  }

  if (!PLATFORM_MAP[platform][arch]) {
    console.error(`Unsupported architecture: ${arch} on ${platform}`);
    console.error(`Supported architectures for ${platform}:`, Object.keys(PLATFORM_MAP[platform]).join(', '));
    throw new Error(`Unsupported architecture: ${arch}`);
  }

  return {
    platform,
    arch,
    platformKey: PLATFORM_MAP[platform][arch],
    isWindows: platform === 'win32'
  };
}

function getDownloadUrl(platformKey, version) {
  const fileName = `rustdex-${version}-${platformKey}.zip`;
  return `https://github.com/${GITHUB_REPO}/releases/download/v${version}/${fileName}`;
}

function getBinaryName(isWindows) {
  return isWindows ? 'rustdex.exe' : 'rustdex';
}

function downloadFile(url, destPath, retries = 3) {
  return new Promise((resolve, reject) => {
    const protocol = url.startsWith('https') ? https : http;

    const doDownload = (attempt) => {
      console.log(`Downloading: ${url}`);

      protocol.get(url, (response) => {
        // Handle redirects
        if (response.statusCode === 302 || response.statusCode === 301 || response.statusCode === 307 || response.statusCode === 308) {
          const redirectUrl = response.headers.location;
          console.log(`Redirecting to: ${redirectUrl}`);
          return downloadFile(redirectUrl, destPath, retries)
            .then(resolve)
            .catch(reject);
        }

        // Handle errors
        if (response.statusCode === 404) {
          reject(new Error(`Release not found (404). Please verify that v${VERSION} exists at https://github.com/${GITHUB_REPO}/releases`));
          return;
        }

        if (response.statusCode !== 200) {
          reject(new Error(`Failed to download: HTTP ${response.statusCode} ${response.statusMessage}`));
          return;
        }

        // Download file
        const file = fs.createWriteStream(destPath);
        response.pipe(file);

        file.on('finish', () => {
          file.close();
          resolve();
        });

        file.on('error', (err) => {
          fs.unlink(destPath, () => {});
          reject(err);
        });
      }).on('error', (err) => {
        if (attempt < retries) {
          console.log(`Download failed (attempt ${attempt + 1}/${retries}), retrying...`);
          setTimeout(() => doDownload(attempt + 1), 1000);
        } else {
          fs.unlink(destPath, () => {});
          reject(new Error(`Download failed after ${retries} attempts: ${err.message}`));
        }
      });
    };

    doDownload(1);
  });
}

function extractZip(zipPath, destDir) {
  return new Promise((resolve, reject) => {
    console.log('Extracting binary...');

    const isWindows = os.platform() === 'win32';

    if (isWindows) {
      // Windows: try adm-zip first, then PowerShell
      try {
        const AdmZip = require('adm-zip');
        const zip = new AdmZip(zipPath);
        zip.extractAllTo(destDir, true);
        resolve();
      } catch (err) {
        if (err.code === 'MODULE_NOT_FOUND') {
          console.log('adm-zip not available, using PowerShell...');
        } else {
          console.log('adm-zip failed, trying PowerShell...');
        }

        try {
          // Escape paths for PowerShell
          const escapedZipPath = zipPath.replace(/'/g, "''");
          const escapedDestDir = destDir.replace(/'/g, "''");

          const psCommand = `
            $ErrorActionPreference = 'Stop'
            $shell = New-Object -ComObject Shell.Application
            $zip = $shell.Namespace('${escapedZipPath}')
            $dest = $shell.Namespace('${escapedDestDir}')
            $dest.CopyHere($zip.Items(), 0x4 + 0x10 + 0x400)
          `;
          execSync(`powershell -NoProfile -Command "${psCommand}"`, { stdio: 'inherit' });
          resolve();
        } catch (psErr) {
          reject(new Error('Failed to extract zip. Please install adm-zip or ensure PowerShell is available.'));
        }
      }
    } else {
      // Unix-like systems: use unzip command
      try {
        execSync(`unzip -o "${zipPath}" -d "${destDir}"`, { stdio: 'inherit' });
        resolve();
      } catch (err) {
        // Check if unzip is available
        try {
          execSync('which unzip', { stdio: 'pipe' });
          // unzip exists but failed
          reject(new Error('Failed to extract zip file. The file may be corrupted.'));
        } catch (whichErr) {
          // unzip not installed
          reject(new Error('unzip is not installed. Please install it:\n' +
            '  Ubuntu/Debian: sudo apt-get install unzip\n' +
            '  macOS: brew install unzip\n' +
            '  RHEL/CentOS: sudo yum install unzip'));
        }
      }
    }
  });
}

function makeExecutable(filePath) {
  if (os.platform() !== 'win32') {
    try {
      fs.chmodSync(filePath, '755');
    } catch (err) {
      console.warn(`Warning: Could not set executable permission on ${filePath}: ${err.message}`);
    }
  }
}

async function install() {
  let tempDir = null;

  try {
    console.log('Installing RustDex...');
    const platformInfo = getPlatformInfo();
    console.log(`Platform: ${platformInfo.platform} ${platformInfo.arch} (${platformInfo.platformKey})`);
    console.log(`Version: ${VERSION}`);

    const binaryName = getBinaryName(platformInfo.isWindows);

    // Setup paths
    const npmBinDir = path.join(__dirname, 'bin');
    const tempBaseDir = path.join(__dirname, '.tmp-install');
    const tempId = Date.now().toString(36);
    tempDir = path.join(tempBaseDir, tempId);
    const zipPath = path.join(tempDir, 'rustdex.zip');
    const binaryPath = path.join(npmBinDir, binaryName);

    // Create directories
    fs.mkdirSync(tempDir, { recursive: true });
    fs.mkdirSync(npmBinDir, { recursive: true });

    // Download zip
    const downloadUrl = getDownloadUrl(platformInfo.platformKey, VERSION);
    await downloadFile(downloadUrl, zipPath);

    // Extract
    await extractZip(zipPath, npmBinDir);

    // Verify extraction
    if (!fs.existsSync(binaryPath)) {
      throw new Error(`Binary not found after extraction. Expected: ${binaryPath}`);
    }

    // Make executable on Unix
    if (!platformInfo.isWindows) {
      makeExecutable(binaryPath);
    }

    // Cleanup temp directory
    fs.rmSync(tempBaseDir, { recursive: true, force: true });

    console.log('');
    console.log('✓ RustDex installed successfully!');
    console.log(`  Binary location: ${binaryPath}`);
    console.log('');
    console.log('To verify installation, run:');
    console.log('  rustdex --version');
    console.log('  rustdex --help');
    console.log('');

  } catch (error) {
    // Cleanup temp directory on error
    if (tempDir && fs.existsSync(tempDir)) {
      try {
        fs.rmSync(tempDir, { recursive: true, force: true });
      } catch (cleanupErr) {
        // Ignore cleanup errors
      }
    }

    console.error('');
    console.error('✗ Installation failed');
    console.error(`  ${error.message}`);
    console.error('');
    console.error('Troubleshooting:');
    console.error('  1. Check your internet connection');
    console.error(`  2. Verify the release exists: https://github.com/${GITHUB_REPO}/releases/tag/v${VERSION}`);
    console.error('  3. Ensure your platform is supported (see supported platforms below)');
    console.error('  4. Try installing with --force flag: npm install -g rustdex --force');
    console.error('');
    console.error('Supported platforms:');
    console.error('  - macOS: Apple Silicon (arm64), Intel (x64)');
    console.error('  - Linux: ARM64 (arm64), AMD64 (x64)');
    console.error('  - Windows: ARM64 (arm64), AMD64 (x64)');
    console.error('');

    process.exit(1);
  }
}

// Run installation
install();