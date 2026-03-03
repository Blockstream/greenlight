import { Node } from '../index.js';
import * as fs from 'fs';
import * as path from 'path';
import { execSync, spawn } from 'child_process';

// Bitcoin CLI helper
export const bitcoinCli = (cmd: any) => execSync(`bitcoin-cli -regtest -datadir=/tmp/gltests/gl-testserver ${cmd}`).toString().trim();

let timeoutMs = 30000;
let checkIntervalMs = 1000;

// LSPS Server configuration
const LSP_DATA_DIR = '/tmp/gltests/lsp-data';
const BITCOIN_CONF = '/tmp/gltests/gl-testserver/bitcoin.conf';
const POLICY_PLUGIN_PATH = '/tmp/gltests/lsps2_policy.py';
let lspProcess: any = null;
let lspSecret: string = '';
export const lspCli = (cmd: any) => execSync(`lightning-cli --network=regtest --lightning-dir=${LSP_DATA_DIR} ${cmd}`).toString().trim();

/**
 * Start an LSPS server for testing
 */
export async function startLspServer(): Promise<{ dataDir: string; rpcSocket: string; secret: string }> {
  console.log('\x1b[32m%s\x1b[0m', 'Starting LSPS server setup...');

  try {
    // Step 1: Ensure bitcoin.conf exists
    ensureBitcoinConf();
    
    // Step 2: Extract RPC config
    const { rpcPort, rpcUser, rpcPass } = getBitcoinRpcConfig();
    console.log(`\x1b[32m%s\x1b[0m`, `Using Bitcoin RPC: ${rpcUser}@localhost:${rpcPort}`);
    
    // Step 3: Ensure policy plugin exists
    ensurePolicyPlugin();
    
    // Step 4: Create LSP data directory
    fs.mkdirSync(LSP_DATA_DIR, { recursive: true });
    fs.mkdirSync(path.join(LSP_DATA_DIR, 'regtest'), { recursive: true });
    
    // Step 5: Generate LSPS secret
    lspSecret = generateLspSecret();
    console.log(`\x1b[32m%s\x1b[0m`, `LSP data directory: ${LSP_DATA_DIR}`);
    console.log(`\x1b[32m%s\x1b[0m`, `LSPS Secret: ${lspSecret}`);
    
    // Step 6: Kill any existing process
    stopLspServer();
    
    // Step 7: Start lightningd process
    await startLightningProcess(rpcPort, rpcUser, rpcPass);
    
    // Step 8: Wait for node to be ready
    await waitForLspNode();
    
    console.log('\x1b[32m%s\x1b[0m', '=== LSPS Server Started Successfully ===');
    console.log(`Data directory: ${LSP_DATA_DIR}`);
    console.log(`RPC socket: ${path.join(LSP_DATA_DIR, 'regtest', 'lightning-rpc')}`);
    console.log(`Bitcoin RPC: ${rpcUser}@localhost:${rpcPort}`);
    console.log(`LSPS Secret: ${lspSecret}`);
    
    return {
      dataDir: LSP_DATA_DIR,
      rpcSocket: path.join(LSP_DATA_DIR, 'regtest', 'lightning-rpc'),
      secret: lspSecret
    };
    
  } catch (error) {
    console.error('\x1b[31m%s\x1b[0m', 'Error starting LSPS server:', error);
    throw error;
  }
}

/**
 * Stop the LSPS server
 */
export function stopLspServer(): void {
  console.log('Stopping LSPS server...');
  
  // Check PID file
  const pidFile = path.join(LSP_DATA_DIR, 'lightningd.pid');
  let stopped = false;
  
  if (fs.existsSync(pidFile)) {
    try {
      const pid = parseInt(fs.readFileSync(pidFile, 'utf8').trim());
      console.log(`Found PID ${pid} from PID file`);
      
      try {
        process.kill(pid, 'SIGTERM');
        console.log(`Sent SIGTERM to process ${pid}`);
        stopped = true;
      } catch (e: any) {
        if (e.code === 'ESRCH') {
          console.log(`Process ${pid} not found`);
        } else {
          console.log(`Error killing process: ${e.message}`);
        }
      }
      
      // Remove PID file
      fs.unlinkSync(pidFile);
    } catch (e) {
      console.log('Error reading PID file:', e);
    }
  }
  
  // If not stopped by PID, try pkill
  if (!stopped) {
    try {
      execSync(`pkill -f "lightning-dir=${LSP_DATA_DIR}"`);
      console.log('Killed processes by command line');
      stopped = true;
    } catch (e) {
      console.log('No processes found to kill');
    }
  }
  
  // Also kill the stored process reference
  if (lspProcess) {
    try {
      lspProcess.kill();
      lspProcess = null;
    } catch (e) {
      // Ignore
    }
  }
  
  console.log('LSPS server stopped');
}

/**
 * Ensure bitcoin.conf exists with proper configuration
 */
function ensureBitcoinConf(): void {
  if (!fs.existsSync(BITCOIN_CONF)) {
    console.log('\x1b[33m%s\x1b[0m', `Bitcoin config not found at ${BITCOIN_CONF}`);
    console.log('Creating directory and config file...');
    
    const confDir = path.dirname(BITCOIN_CONF);
    fs.mkdirSync(confDir, { recursive: true });
    
    const confContent = `regtest=1
rpcuser=rpcuser
rpcpassword=rpcpass
fallbackfee=0.00001
rpcport=34917
[regtest]
rpcport=34917
`;
    
    fs.writeFileSync(BITCOIN_CONF, confContent);
    console.log('\x1b[32m%s\x1b[0m', 'Created bitcoin.conf');
  }
}

/**
 * Extract Bitcoin RPC configuration from bitcoin.conf
 */
function getBitcoinRpcConfig(): { rpcPort: string; rpcUser: string; rpcPass: string } {
  const confContent = fs.readFileSync(BITCOIN_CONF, 'utf8');
  const lines = confContent.split('\n');
  
  let rpcPort = '34917';
  let rpcUser = 'rpcuser';
  let rpcPass = 'rpcpass';
  
  for (const line of lines) {
    if (line.startsWith('rpcport=')) {
      rpcPort = line.split('=')[1].trim();
    } else if (line.startsWith('rpcuser=')) {
      rpcUser = line.split('=')[1].trim();
    } else if (line.startsWith('rpcpassword=')) {
      rpcPass = line.split('=')[1].trim();
    }
  }
  
  return { rpcPort, rpcUser, rpcPass };
}

/**
 * Ensure LSPS policy plugin exists
 */
function ensurePolicyPlugin(): void {
  if (!fs.existsSync(POLICY_PLUGIN_PATH)) {
    console.log('\x1b[33m%s\x1b[0m', 'Policy plugin not found. Creating default lsps2_policy.py...');
    
    const pluginDir = path.dirname(POLICY_PLUGIN_PATH);
    fs.mkdirSync(pluginDir, { recursive: true });
    
    const pluginContent = `#!/usr/bin/env python3
"""A simple implementation of a LSPS2 compatible policy plugin."""
from pyln.client import Plugin
from datetime import datetime, timedelta, timezone

plugin = Plugin()

@plugin.method("lsps2-policy-getpolicy")
def lsps2_policy_getpolicy(request):
    """Returns an opening fee menu for the LSPS2 plugin."""
    now = datetime.now(timezone.utc)
    valid_until = (now + timedelta(hours=1)).isoformat().replace("+00:00", "Z")

    return {
        "policy_opening_fee_params_menu": [
            {
                "min_fee_msat": "1000",
                "proportional": 1000,
                "valid_until": valid_until,
                "min_lifetime": 2000,
                "max_client_to_self_delay": 2016,
                "min_payment_size_msat": "1000",
                "max_payment_size_msat": "100000000",
            },
            {
                "min_fee_msat": "1092000",
                "proportional": 2400,
                "valid_until": valid_until,
                "min_lifetime": 1008,
                "max_client_to_self_delay": 2016,
                "min_payment_size_msat": "1000",
                "max_payment_size_msat": "1000000",
            },
        ]
    }

@plugin.method("lsps2-policy-getchannelcapacity")
def lsps2_policy_getchannelcapacity(request, init_payment_size, scid, opening_fee_params):
    """Returns channel capacity."""
    return {"channel_capacity_msat": 100000000}

plugin.run()
`;
    
    fs.writeFileSync(POLICY_PLUGIN_PATH, pluginContent);
    fs.chmodSync(POLICY_PLUGIN_PATH, 0o755); // Make executable
    console.log('\x1b[32m%s\x1b[0m', 'Created policy plugin');
  }
}

/**
 * Generate a random LSPS secret (64 hex chars)
 */
function generateLspSecret(): string {
  const crypto = require('crypto');
  return crypto.randomBytes(32).toString('hex');
}

/**
 * Start the lightningd process
 */
async function startLightningProcess(rpcPort: string, rpcUser: string, rpcPass: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const args = [
      `--lightning-dir=${LSP_DATA_DIR}`,
      '--network=regtest',
      '--experimental-lsps2-service',
      `--experimental-lsps2-promise-secret=${lspSecret}`,
      `--important-plugin=${POLICY_PLUGIN_PATH}`,
      '--disable-plugin=cln-grpc',
      '--bitcoin-rpcconnect=127.0.0.1',
      `--bitcoin-rpcport=${rpcPort}`,
      `--bitcoin-rpcuser=${rpcUser}`,
      `--bitcoin-rpcpassword=${rpcPass}`,
      '--log-level=debug',
      `--log-file=${path.join(LSP_DATA_DIR, 'log')}`,
      '--addr=127.0.0.1:9735'
    ];
    
    console.log('\x1b[32m%s\x1b[0m', 'Starting lightningd as LSPS server...');
    console.log(`Command: lightningd ${args.join(' ')}`);
    
    lspProcess = spawn('lightningd', args, {
      detached: false,
      stdio: 'pipe'
    });
    
    // Save PID
    if (lspProcess.pid) {
      fs.writeFileSync(path.join(LSP_DATA_DIR, 'lightningd.pid'), lspProcess.pid.toString());
      console.log(`\x1b[32m%s\x1b[0m`, `Lightningd started with PID: ${lspProcess.pid}`);
    }
    
    // Log output
    lspProcess.stdout.on('data', (data: any) => {
      console.log(`[lightningd stdout]: ${data.toString().trim()}`);
    });
    
    lspProcess.stderr.on('data', (data: any) => {
      console.error(`[lightningd stderr]: ${data.toString().trim()}`);
    });
    
    lspProcess.on('error', (err: any) => {
      console.error('\x1b[31m%s\x1b[0m', 'Failed to start lightningd:', err);
      reject(err);
    });
    
    // Don't wait for process to exit - it should keep running
    resolve();
  });
}

/**
 * Wait for LSP node to be ready (RPC socket available)
 */
async function waitForLspNode(maxWaitSeconds: number = 30): Promise<void> {
  console.log('\x1b[33m%s\x1b[0m', 'Waiting for node to start...');
  
  const rpcSocket = path.join(LSP_DATA_DIR, 'regtest', 'lightning-rpc');
  const startTime = Date.now();
  
  while (Date.now() - startTime < maxWaitSeconds * 1000) {
    if (fs.existsSync(rpcSocket)) {
      try {
        // Try to connect to test if it's ready
        const result = execSync(`lightning-cli --network=regtest --lightning-dir=${LSP_DATA_DIR} getinfo 2>/dev/null || true`).toString();
        if (result) {
          console.log('\x1b[32m%s\x1b[0m', `Node is ready! RPC socket: ${rpcSocket}`);
          
          // Test LSPS policy
          try {
            const policyResult = execSync(`lightning-cli --network=regtest --lightning-dir=${LSP_DATA_DIR} lsps2-policy-getpolicy 2>/dev/null`).toString();
            console.log('\x1b[32m%s\x1b[0m', 'LSPS policy test successful');
          } catch (e) {
            console.log('\x1b[33m%s\x1b[0m', 'Could not test policy (may need to wait longer)');
          }
          
          return;
        }
      } catch (e) {
        // Socket exists but node not fully ready
      }
    }
    
    await new Promise(resolve => setTimeout(resolve, 2000));
    const elapsed = Math.floor((Date.now() - startTime) / 1000);
    console.log(`Waiting for RPC socket... (${elapsed}/${maxWaitSeconds} seconds)`);
  }
  
  throw new Error(`Node failed to start properly within ${maxWaitSeconds} seconds`);
}

/**
 * Check if LSPS server is running
 */
export function isLspServerRunning(): boolean {
  try {
    execSync(`pgrep -f "lightning-dir=${LSP_DATA_DIR}"`);
    return true;
  } catch (e) {
    return false;
  }
}

/**
 * Get LSPS server info
 */
export function getLspServerInfo(): { dataDir: string; pidFile: string; rpcSocket: string } {
  return {
    dataDir: LSP_DATA_DIR,
    pidFile: path.join(LSP_DATA_DIR, 'lightningd.pid'),
    rpcSocket: path.join(LSP_DATA_DIR, 'regtest', 'lightning-rpc')
  };
}

interface FundNodeOptions {
  amountBtc?: number;
  timeoutMs?: number;
  checkIntervalMs?: number;
  requiredConfirmations?: number;
  waitForSpecificTx?: boolean;
}

interface FundNodeResult {
  txid: string;
  address: string;
  amountBtc: number;
  detected: boolean;
  outputs?: any[];
  totalSats?: number;
  timeElapsedMs: number;
}

export async function fundNode(
  node: Node, 
  amountBtc = 1,
  options: FundNodeOptions = {}
): Promise<FundNodeResult> {
  const requiredConfirmations = options.requiredConfirmations || 4;
  const waitForSpecificTx = options.waitForSpecificTx !== false;
  timeoutMs = options.timeoutMs || timeoutMs;
  checkIntervalMs = options.checkIntervalMs || checkIntervalMs;
  
  const startTime = Date.now();
  
  try {
    // Ensure we have enough blocks for coinbase maturity
    const blockCount = parseInt(bitcoinCli('getblockcount'));
    if (blockCount < 101) {
      const minerAddress = bitcoinCli('getnewaddress');
      bitcoinCli(`generatetoaddress ${101 - blockCount} ${minerAddress}`);
      console.log(`⛏️  Generated blocks to reach block 101 (current: ${blockCount})`);
    }
    
    // Get node's address
    const nodeAddress = (await node.onchainReceive()).bech32;
    console.log(`💰 Funding node at address: ${nodeAddress} with ${amountBtc} BTC`);
    
    // Send funds to node
    const txid = bitcoinCli(`sendtoaddress ${nodeAddress} ${amountBtc}`);
    console.log(`📤 Transaction sent: ${txid}`);
    
    // Generate blocks to confirm
    bitcoinCli(`-generate ${requiredConfirmations}`);
    console.log(`⛏️  Generated ${requiredConfirmations} blocks to confirm transaction`);
    
    // Wait for node to detect the funds by checking listFunds.outputs
    console.log(`⏳ Waiting for node to detect funds (timeout: ${timeoutMs}ms)...`);
    
    let lastOutputCount = 0;
    let funded = false;
    let detectedOutputs: any[] = [];
    
    while (Date.now() - startTime < timeoutMs) {
      try {
        // Check node's funds using listFunds
        const funds = await node.listFunds();
        const outputs = funds.outputs || [];
        
        // Log progress if new outputs appear
        if (outputs.length !== lastOutputCount) {
          console.log(`📊 Node has ${outputs.length} UTXOs total`);
          lastOutputCount = outputs.length;
        }
        
        // Look for our specific transaction in confirmed outputs
        const ourOutput = outputs.find((output: any) => {
          const txidStr = Buffer.isBuffer(output.txid) ? output.txid.toString('hex') : output.txid;
          return (txidStr === txid && output.status === 'confirmed') ? output : null;
        });
        
        if (ourOutput && waitForSpecificTx) {
          const amountSats = ourOutput.amountMsat / 1000;
          console.log(`✅ Found our specific transaction!`);
          console.log(`   Amount: ${amountSats} sats (${amountSats/100000000} BTC)`);
          console.log(`   Status: ${ourOutput.status}`);
          console.log(`   Address: ${ourOutput.address}`);
          detectedOutputs = [ourOutput];
          funded = true;
          break;
        }

        // Print detailed output info occasionally for debugging
        if (outputs.length > 0 && Date.now() - startTime > 5000 && outputs.length <= 3) {
          console.log('📋 Current UTXOs in listFunds.outputs:');
          outputs.forEach((out: any, i: number) => {
            const amountSats = out.amountMsat / 1000;
            console.log(`   ${i+1}. ${amountSats} sats - ${out.status}${out.reserved ? ' (reserved)' : ''}`);
            if (out.txid) {
              const txidStr = Buffer.isBuffer(out.txid) ? out.txid.toString('hex') : out.txid;
              console.log(`      txid: ${txidStr.substring(0,16)}...`);
            }
          });
        }
        await new Promise(resolve => setTimeout(resolve, checkIntervalMs));
      } catch (error) {
        console.log('⚠️  Error checking funds, retrying:', error);
        await new Promise(resolve => setTimeout(resolve, checkIntervalMs));
      }
    }
    
    if (!funded) {
      // Final check to see what we have
      const funds = await node.listFunds().catch(() => ({ outputs: [] }));
      const outputs = funds.outputs || [];
      const totalSats = outputs.reduce(
        (sum: number, o: any) => sum + parseInt(o.amountMsat), 0
      ) / 1000;
      
      throw new Error(
        `⛔ Timeout after ${timeoutMs}ms waiting for node to detect funds. ` +
        `Node has ${outputs.length} UTXOs totaling ${totalSats} sats. ` +
        `Expected at least ${amountBtc * 100000000} sats.`
      );
    }
    
    const timeElapsedMs = Date.now() - startTime;
    const totalSats = detectedOutputs.reduce(
      (sum: number, o: any) => sum + parseInt(o.amountMsat), 0
    ) / 1000;
    
    console.log(`✅ Node successfully funded in ${timeElapsedMs}ms`);
    console.log(`   Address: ${nodeAddress}`);
    console.log(`   Total: ${totalSats} sats (${totalSats/100000000} BTC)`);
    console.log(`   UTXOs: ${detectedOutputs.length}`);
    console.log(`   Txid: ${txid}`);
    
    return {
      txid,
      address: nodeAddress,
      amountBtc,
      detected: true,
      outputs: detectedOutputs,
      totalSats,
      timeElapsedMs
    };
    
  } catch (error: any) {
    console.error('❌ Error funding node:', error?.message ?? error);
    throw error;
  }
}
