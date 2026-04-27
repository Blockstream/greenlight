import * as fs from 'fs';
import * as crypto from 'crypto';
import * as bip39 from 'bip39';
import { Config, Credentials, Scheduler, Node, register, connect } from '../index.js';

export const lspInfo = () => ({
  rpcSocket:     process.env.LSP_RPC_SOCKET!,
  nodeId:        process.env.LSP_NODE_ID!,
});

export async function fundWallet(node: Node, amountSats = 100_000_000): Promise<boolean> {
  const testSetupServerUrl = process.env.TEST_SETUP_SERVER_URL!;
  if (!testSetupServerUrl) throw new Error('TEST_SETUP_SERVER_URL not set');

  const address = (await node.onchainReceive()).bech32;
  const res = await fetch(`${testSetupServerUrl}/fund-wallet`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ address, amount: amountSats }),
  });

  if (!res.ok) {
    const error = await res.text();
    throw new Error(`fundNode failed (${res.status}): ${error}`);
  }

  // Wait for node to detect the confirmed UTXO
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    const funds = await node.listFunds();
    if ((funds.outputs ?? []).length > 0) return true;
    await new Promise((r) => setTimeout(r, 1_000));
  }

  throw new Error('fundNode timed out waiting for node to detect funds');
}

/** A fully-connected SDK node with the SDK signer running. The SDK
 *  manages the signer internally; call `node.disconnect()` to stop
 *  it. The `mnemonic` is returned so callers that need cryptographic
 *  derivations (e.g. LNURL-auth happens internally on Node, but tests
 *  may want to assert against the same seed) can re-use it. */
export interface SdkNode {
  node: Node;
  mnemonic: string;
}

export async function getGLNode(
  _scheduler: Scheduler,
  connectToLSP: boolean = true,
): Promise<SdkNode> {
  const mnemonic = bip39.entropyToMnemonic(crypto.randomBytes(16).toString('hex'));
  const config = new Config().withNetwork('regtest');

  if (connectToLSP) {
    const testSetupServerUrl = process.env.TEST_SETUP_SERVER_URL!;
    if (!testSetupServerUrl) throw new Error('TEST_SETUP_SERVER_URL not set');

    // Test-setup server registers a node with the same seed and links
    // it to the LSP. We then `connect` from the JS side using the
    // mnemonic that derives that same seed.
    const secret = bip39.mnemonicToSeedSync(mnemonic);
    const res = await fetch(`${testSetupServerUrl}/connect-to-lsp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ secret: secret.toString('hex') }),
    });
    if (!res.ok) throw new Error(`Failed to connect node to LSP: ${await res.text()}`);
    const { creds_path } = await res.json();
    const credsBytes = fs.readFileSync(creds_path);
    const node = await connect(mnemonic, credsBytes, config);
    return { node, mnemonic };
  }

  const node = await register(mnemonic, undefined, config);
  return { node, mnemonic };
}

export async function getLspInvoice(amountMsat: number = 0): Promise<string> {
  const testSetupServerUrl = process.env.TEST_SETUP_SERVER_URL!;
  if (!testSetupServerUrl) throw new Error('TEST_SETUP_SERVER_URL not set');
  const res = await fetch(`${testSetupServerUrl}/lsp-invoice`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ amount_msat: amountMsat, label: `test-${Date.now()}`, description: 'Test payment' }),
  });
  if (!res.ok) throw new Error(`Failed to get LSP invoice: ${await res.text()}`);
  const { bolt11 } = await res.json();
  return bolt11;
}

export async function getBitcoinAddress(): Promise<string> {
  const testSetupServerUrl = process.env.TEST_SETUP_SERVER_URL!;
  if (!testSetupServerUrl) throw new Error('TEST_SETUP_SERVER_URL not set');
  const res = await fetch(`${testSetupServerUrl}/btc-address`, { method: 'POST' });
  if (!res.ok) throw new Error(`Failed to get Bitcoin address: ${await res.text()}`);
  const resJson = await res.json();
  return resJson.address;
}

// Re-export `Credentials` so tests that need to load credentials
// directly (without going through register/connect) keep compiling.
export { Credentials };
