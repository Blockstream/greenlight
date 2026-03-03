import * as crypto from 'crypto';
import * as bip39 from 'bip39';
import { Credentials, Scheduler, Signer, Node } from '../index.js';

describe('Greenlight node', () => {
  let node: Node;
  let credentials: Credentials;

  it('can be setup', async () => {
    const rand: Buffer = crypto.randomBytes(16);
    const MNEMONIC: string = bip39.entropyToMnemonic(rand.toString("hex"));
    const scheduler = new Scheduler('regtest');
    const signer = new Signer(MNEMONIC);
    const nodeId = signer.nodeId();

    expect(Buffer.isBuffer(nodeId)).toBe(true);
    expect(nodeId.length).toBeGreaterThan(0);

    credentials = await scheduler.register(signer);
    node = new Node(credentials);
    expect(node).toBeTruthy();
  });
});
