import { Node } from '../index.js';

export const lspInfo = () => ({
  rpcSocket:     process.env.LSP_RPC_SOCKET!,
  nodeId:        process.env.LSP_NODE_ID!,
  promiseSecret: process.env.LSP_PROMISE_SECRET!,
});

export async function fundWallet(node: Node, amountSats = 100_000_000): Promise<boolean> {
  const address = (await node.onchainReceive()).bech32;
  const res = await fetch(process.env.GL_FUND_SERVER!, {
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
