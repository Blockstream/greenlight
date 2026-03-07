import { spawn, ChildProcess } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';
import * as dotenv from 'dotenv';

const verbose = process.argv.includes('--verbose') || process.env.VERBOSE === '1';
const GLTESTS_DIR = '/tmp/gltests';
const ENV_FILE = path.join(GLTESTS_DIR, '.env');
const SERVER_READY_TIMEOUT_MS = 60_000;
const SERVER_READY_POLL_MS = 500;

// Store PID so globalTeardown can kill it
const PID_FILE = path.join(GLTESTS_DIR, 'gltestserver.pid');

/**
 * Wait for the .env file to appear and be fully populated.
 * gltestserver writes this file once it's ready.
 */
async function waitForEnvFile(): Promise<Record<string, string>> {
  const requiredKeys = [
    'GL_SCHEDULER_GRPC_URI',
    'GL_CA_CRT',
    'GL_NOBODY_CRT',
    'GL_NOBODY_KEY',
    'LSP_RPC_SOCKET',
    'LSP_NODE_ID',
    'LSP_PROMISE_SECRET',
    'GL_FUND_SERVER',
  ];

  const deadline = Date.now() + SERVER_READY_TIMEOUT_MS;

  while (Date.now() < deadline) {
    if (fs.existsSync(ENV_FILE)) {
      const parsed = dotenv.parse(fs.readFileSync(ENV_FILE, 'utf8'));
      const missingKeys = requiredKeys.filter((k) => !parsed[k]);

      if (missingKeys.length === 0) {
        return parsed;
      }
    }

    await new Promise((r) => setTimeout(r, SERVER_READY_POLL_MS));
  }

  throw new Error(
    `gltestserver did not write ${ENV_FILE} with required keys within ${SERVER_READY_TIMEOUT_MS}ms.\n` +
      `Required keys: ${requiredKeys.join(', ')}`
  );
}

export default async function globalSetup(): Promise<void> {
  fs.mkdirSync(GLTESTS_DIR, { recursive: true });

  // Remove stale .env so we don't read an old one
  if (fs.existsSync(ENV_FILE)) {
    fs.unlinkSync(ENV_FILE);
  }

  console.log('\n🚀 Starting gltestserver...');

  const server: ChildProcess = spawn(
    'uv',
    ['run', 'python', path.join(__dirname, 'test_setup.py')],
    {
      detached: true,
      stdio: verbose ? ['ignore', 'pipe', 'pipe'] : 'ignore',
    }
  );

  if (!server.pid) {
    throw new Error('Failed to spawn gltestserver (no PID assigned)');
  }

  // Persist PID for teardown
  fs.writeFileSync(PID_FILE, String(server.pid));
  console.log(`   gltestserver PID: ${server.pid}`);

  if (verbose) {
    server.stdout?.on('data', (d: Buffer) => process.stdout.write(`[test_setup] ${d}`));
  }
  server.stderr?.on('data', (d: Buffer) => process.stderr.write(`[test_setup] ${d}`));
  server.on('error', (err) => {
    throw new Error(`gltestserver process error: ${err.message}`);
  });

  // Unref so Jest can exit even if teardown somehow misses the process
  server.unref();

  // Wait until the server has written its .env
  console.log('   Waiting for gltestserver to be ready...');
  const env = await waitForEnvFile();

  // Inject into process.env so Jest workers inherit these values
  for (const [key, value] of Object.entries(env)) {
    process.env[key] = value;
  }

  console.log('✅ gltestserver ready. Environment variables set:');
  for (const key of Object.keys(env)) {
    console.log(`   ${key}=${process.env[key]?.slice(0, 60)}...`);
  }
}
