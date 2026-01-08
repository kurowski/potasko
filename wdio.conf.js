import { spawn, spawnSync } from 'child_process';
import path from 'path';
import fs from 'fs';

let tauriDriver;

// Database paths
const APP_DATA_DIR = path.join(process.env.HOME, '.local/share/com.vscode.potasko');
const APP_DB = path.join(APP_DATA_DIR, 'potasko.db');
const CLI_PATH = './src-tauri/target/debug/potasko';

export const config = {
  specs: ['./tests/e2e/**/*.spec.js'],
  maxInstances: 1,

  // Connect to tauri-driver running on port 4444
  protocol: 'http',
  hostname: '127.0.0.1',
  port: 4444,
  path: '/',

  capabilities: [{
    'tauri:options': {
      application: './src-tauri/target/debug/potasko-gui'
    }
  }],

  framework: 'mocha',
  mochaOpts: {
    timeout: 60000
  },
  reporters: ['spec'],

  // Start tauri-driver and build app before tests
  onPrepare: async () => {
    // Build the app first
    console.log('Building Tauri app (debug)...');
    const result = spawnSync('pnpm', ['tauri', 'build', '--debug', '--no-bundle'], {
      stdio: 'inherit',
      cwd: process.cwd()
    });
    if (result.status !== 0) {
      throw new Error('Failed to build Tauri app');
    }

    // Clean up database BEFORE starting the app
    console.log('Cleaning up database...');
    try {
      if (fs.existsSync(APP_DB)) {
        fs.unlinkSync(APP_DB);
        console.log('Removed existing app database');
      }
      // Initialize database via CLI (runs migrations)
      spawnSync(CLI_PATH, ['--database', APP_DB, 'list', 'list'], { stdio: 'inherit' });
      console.log('Initialized fresh database via CLI');
    } catch (e) {
      console.log('Could not cleanup/init app database:', e.message);
    }

    // Ensure Radicale is running for CalDAV tests
    console.log('Starting Radicale...');
    spawnSync('sudo', ['service', 'radicale', 'start'], { stdio: 'inherit' });

    // Then start tauri-driver
    console.log('Starting tauri-driver...');
    tauriDriver = spawn('tauri-driver', [], {
      stdio: ['ignore', 'pipe', 'pipe']
    });

    // Log tauri-driver output for debugging
    tauriDriver.stdout.on('data', (data) => {
      console.log(`tauri-driver stdout: ${data}`);
    });
    tauriDriver.stderr.on('data', (data) => {
      console.error(`tauri-driver stderr: ${data}`);
    });

    // Wait for tauri-driver to start
    await new Promise((resolve) => setTimeout(resolve, 2000));
    console.log('tauri-driver started');
  },

  // Clean up after all tests
  onComplete: () => {
    console.log('Stopping tauri-driver...');
    if (tauriDriver) {
      tauriDriver.kill();
    }
  }
};
