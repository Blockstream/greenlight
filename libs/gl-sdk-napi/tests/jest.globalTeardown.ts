import * as fs from 'fs';
import * as path from 'path';

const GLTESTS_DIR = '/tmp/gltests';
const PID_FILE = path.join(GLTESTS_DIR, 'gltestserver.pid');
const BITCOIND_PID_FILE = path.join(GLTESTS_DIR, 'gl-testserver', 'regtest', 'bitcoind.pid');

async function stop_process(process_name: string, pid_file_location: string) {
  console.log(`\n🛑 Stopping ${process_name}...`);

  if (!fs.existsSync(pid_file_location)) {
    console.log(`   No PID file found for ${process_name}, skipping.`);
    return;
  }

  try {
    const pid = parseInt(fs.readFileSync(pid_file_location, 'utf8').trim(), 10);

    if (!isNaN(pid)) {
      try {
        process.kill(pid, 'SIGTERM');
        console.log(`   Sent SIGTERM to ${process_name} (PID ${pid})`);
        await new Promise((r) => setTimeout(r, 2_000));
        process.kill(pid, 'SIGKILL');
      } catch {
        // Process already exited — that's fine
      }
    }

    if (fs.existsSync(pid_file_location)) {
      fs.unlinkSync(pid_file_location);
    }
  } catch {
    // Ignore all teardown errors
  }

  console.log(`✅ ${process_name} stopped.`);
}

export default async function globalTeardown(): Promise<void> {
  await stop_process('gltestserver', PID_FILE);
  await stop_process('bitcoind', BITCOIND_PID_FILE);
}
