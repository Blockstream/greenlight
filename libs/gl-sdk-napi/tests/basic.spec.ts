import * as crypto from 'crypto';
import * as bip39 from 'bip39';
import { Credentials, Scheduler, Signer, Node } from '../index.js';

describe('Greenlight node', () => {
  it('can be setup', async () => {
    const scheduler = new Scheduler('regtest');
    const rand: Buffer = crypto.randomBytes(16);
    const MNEMONIC: string = bip39.entropyToMnemonic(rand.toString("hex"));
    const signer = new Signer(MNEMONIC);
    const handle = await signer.start();
    const nodeId = signer.nodeId();
    expect(Buffer.isBuffer(nodeId)).toBe(true);
    expect(nodeId.length).toBeGreaterThan(0);
    const credentials = await scheduler.register(signer);
    const node = new Node(credentials);
    expect(node).toBeTruthy();
    handle.stop();
    await node.stop();
  });
});
