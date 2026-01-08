// E2E Test Setup Helpers
import { execSync } from 'child_process';
import path from 'path';
import fs from 'fs';

const PROJECT_ROOT = path.resolve(import.meta.dirname, '../../');
const CLI_PATH = path.join(PROJECT_ROOT, 'src-tauri/target/debug/potasko');
const APP_DATA_DIR = path.join(process.env.HOME, '.local/share/com.vscode.potasko');
const APP_DB = path.join(APP_DATA_DIR, 'potasko.db');

/**
 * Clean up the app's database and reinitialize it.
 * Uses CLI to ensure migrations run before the GUI starts.
 */
export function cleanupAppDatabase() {
  try {
    if (fs.existsSync(APP_DB)) {
      fs.unlinkSync(APP_DB);
      console.log('Removed existing app database');
    }
    // Initialize database via CLI (runs migrations)
    execSync(`${CLI_PATH} --database "${APP_DB}" list list`, { stdio: 'inherit' });
    console.log('Initialized fresh database via CLI');
  } catch (e) {
    console.log('Could not cleanup/init app database:', e.message);
  }
}

/**
 * Ensure Radicale is running for CalDAV tests.
 */
export function ensureRadicaleRunning() {
  try {
    execSync('sudo service radicale start', { stdio: 'inherit' });
  } catch (e) {
    console.log('Radicale may already be running');
  }
}

/**
 * Set up a test CalDAV account via CLI using the app's database.
 */
export async function setupTestAccount() {
  ensureRadicaleRunning();
  cleanupAppDatabase();

  // The app will initialize the database on first run
  // We'll add the account via CLI after the app starts
}

/**
 * Run CLI command using the app's database.
 */
export function runCli(args) {
  return execSync(`${CLI_PATH} --database "${APP_DB}" ${args}`, {
    encoding: 'utf-8'
  });
}
