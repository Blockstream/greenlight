import * as fs from 'fs';
import * as crypto from 'crypto';
import * as bip39 from 'bip39';
import { Credentials, Scheduler, Signer, Node, Handle } from '../index.js';

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

export async function getGLNode(scheduler: Scheduler, connectToLSP: boolean = true): Promise<{ node: Node; handle: Handle }> {
  const mnemonic = bip39.entropyToMnemonic(crypto.randomBytes(16).toString('hex'));
  const secret = bip39.mnemonicToSeedSync(mnemonic);
  const signer = new Signer(mnemonic);
  let credentials: Credentials;
  if (connectToLSP) {
    const testSetupServerUrl = process.env.TEST_SETUP_SERVER_URL!;
    if (!testSetupServerUrl) throw new Error('TEST_SETUP_SERVER_URL not set');

    const res = await fetch(`${testSetupServerUrl}/connect-to-lsp`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ secret: secret.toString('hex') }),
    });
    if (!res.ok) throw new Error(`Failed to connect node to LSP: ${await res.text()}`);
    const { creds_path } = await res.json();
    credentials = await Credentials.load(fs.readFileSync(creds_path));
  } else {
    credentials = await scheduler.register(signer);
  }
  return { node: new Node(credentials), handle: await signer.start() };
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