#!/usr/bin/env node

/**
 * RustDex Binary Uninstaller
 * Removes the downloaded binary
 */

const fs = require('fs');
const path = require('path');
const os = require('os');

function getBinaryName(isWindows) {
  return isWindows ? 'rustdex.exe' : 'rustdex';
}

function uninstall() {
  try {
    console.log('Uninstalling RustDex...');

    const isWindows = os.platform() === 'win32';
    const binaryName = getBinaryName(isWindows);

    const npmBinDir = path.join(__dirname, 'bin');
    const binaryPath = path.join(npmBinDir, binaryName);
    const wrapperPath = path.join(npmBinDir, 'rustdex' + (isWindows ? '.cmd' : ''));

    // Remove binary
    if (fs.existsSync(binaryPath)) {
      fs.unlinkSync(binaryPath);
      console.log(`✓ Removed binary: ${binaryPath}`);
    }

    // Remove wrapper
    if (fs.existsSync(wrapperPath)) {
      fs.unlinkSync(wrapperPath);
      console.log(`✓ Removed wrapper: ${wrapperPath}`);
    }

    console.log('RustDex uninstalled successfully');

  } catch (error) {
    console.error('✗ Uninstallation failed:', error.message);
    process.exit(1);
  }
}

// Run uninstallation
uninstall();