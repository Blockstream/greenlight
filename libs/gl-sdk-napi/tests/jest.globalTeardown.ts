import * as fs from 'fs';
import * as path from 'path';

const GLTESTS_DIR = '/tmp/gltests';
const PID_FILE = path.join(GLTESTS_DIR, 'gltestserver.pid');

export default async function globalTeardown(): Promise<void> {
  console.log(`\n🛑 Stopping test_setup...`);

  if (!fs.existsSync(PID_FILE)) {
    console.log(`   No PID file found for test_setup, skipping.`);
    return;
  }

  try {
    const pid = parseInt(fs.readFileSync(PID_FILE, 'utf8').trim(), 10);

    if (!isNaN(pid)) {
      try {
        process.kill(pid, 'SIGTERM');
        console.log(`   Sent SIGTERM to test_setup (PID ${pid})`);
        await new Promise((r) => setTimeout(r, 2_000));
        process.kill(pid, 'SIGKILL');
      } catch {
        // Process already exited — that's fine
      }
    }

    if (fs.existsSync(PID_FILE)) {
      fs.unlinkSync(PID_FILE);
    }
  } catch {
    // Ignore all teardown errors
  }

  console.log(`✅ test_setup stopped.`);
}
