import { describe, it, expect } from '@jest/globals';
import { Credentials, Scheduler, Signer, Node } from '../index.js';

const MNEMONIC = 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about';

describe('Greenlight node', () => {
  let node: Node;
  let credentials: Credentials;

  it('can be setup', async () => {
    const scheduler = new Scheduler('regtest');
    const signer = new Signer(MNEMONIC);
    const nodeId = signer.nodeId();

    expect(Buffer.isBuffer(nodeId)).toBe(true);
    expect(nodeId.length).toBeGreaterThan(0);

    credentials = await scheduler.recover(signer);
    node = new Node(credentials);
    expect(node).toBeTruthy();
  });
});
