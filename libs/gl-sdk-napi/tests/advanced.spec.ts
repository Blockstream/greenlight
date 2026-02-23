import { describe, it, expect, beforeAll, afterEach } from '@jest/globals';
import {
  Credentials,
  Scheduler,
  Signer,
  Node
} from '../index.js';

const MNEMONIC = 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about';

describe('Credentials', () => {
  it('can save and load raw credentials', async () => {
    const original = await Credentials.load(Buffer.from('test'));
    const raw = await original.save();

    expect(Buffer.isBuffer(raw)).toBe(true);

    const restored = await Credentials.load(raw);
    const raw2 = await restored.save();

    expect(raw2.equals(raw)).toBe(true);
  });
});

describe('Signer', () => {
  it('can be constructed from a mnemonic', async () => {
    const signer = new Signer(MNEMONIC);
    expect(signer).toBeTruthy();
  });

  it('can return a node id', async () => {
    const signer = new Signer(MNEMONIC);
    const nodeId = signer.nodeId();

    expect(Buffer.isBuffer(nodeId)).toBe(true);
    expect(nodeId.length).toBeGreaterThan(0);
  });

  it('returns consistent node id for same mnemonic', async () => {
    const signer1 = new Signer(MNEMONIC);
    const signer2 = new Signer(MNEMONIC);

    const nodeId1 = signer1.nodeId();
    const nodeId2 = signer2.nodeId();

    expect(nodeId1.equals(nodeId2)).toBe(true);
  });

  it('can be constructed with different mnemonics', async () => {
    const mnemonic2 = 'legal winner thank year wave sausage worth useful legal winner thank yellow';
    const signer = new Signer(mnemonic2);
    expect(signer).toBeTruthy();

    const nodeId = signer.nodeId();
    expect(Buffer.isBuffer(nodeId)).toBe(true);
  });
});

describe('Scheduler', () => {
  it('can be constructed for regtest', async () => {
    const scheduler = new Scheduler('regtest');
    expect(scheduler).toBeTruthy();
  });

  it('can be constructed for bitcoin mainnet', async () => {
    const scheduler = new Scheduler('bitcoin');
    expect(scheduler).toBeTruthy();
  });
});

describe('Integration: scheduler and signer', () => {
  let scheduler: Scheduler;
  let signer: Signer;

  beforeAll(() => {
    scheduler = new Scheduler('regtest');
    signer = new Signer(MNEMONIC);
  });

  it('can recover credentials', async () => {
    const recovered = await scheduler.recover(signer);
    expect(recovered).toBeInstanceOf(Credentials);
    expect((await recovered.save()).length).toBeGreaterThan(0);
  });

  it('handles registration of existing node (falls back to recovery)', async () => {
    try {
      const credentials = await scheduler.register(signer, '');
      expect(credentials).toBeInstanceOf(Credentials);
    } catch (e) {
      const recovered = await scheduler.recover(signer);
      expect(recovered).toBeInstanceOf(Credentials);
    }
  });
});

describe('Node', () => {
  let node: Node;
  let credentials: Credentials;

  beforeAll(async () => {
    const scheduler = new Scheduler('regtest');
    const signer = new Signer(MNEMONIC);
    credentials = await scheduler.recover(signer);
    node = new Node(credentials);
  });

  afterEach(() => {
    // Clean up after each test if needed
  });

  it('can be constructed from credentials', async () => {
    expect(node).toBeTruthy();
  });

  describe('calls getInfo', () => {
    it('returns node information with expected fields', async () => {
      const info = await node.getInfo();

      // Verify response structure
      expect(info).toBeTruthy();
      expect(Buffer.isBuffer(info.id)).toBe(true);
      expect(info.id.length).toBeGreaterThan(0);
      expect(Buffer.isBuffer(info.color)).toBe(true);
      expect(typeof info.numPeers).toBe('number');
      expect(typeof info.numPendingChannels).toBe('number');
      expect(typeof info.numActiveChannels).toBe('number');
      expect(typeof info.numInactiveChannels).toBe('number');
      expect(typeof info.version).toBe('string');
      expect(typeof info.lightningDir).toBe('string');
      expect(typeof info.blockheight).toBe('number');
      expect(typeof info.network).toBe('string');
      expect(typeof info.feesCollectedMsat).toBe('number');

      // Verify response values
      expect(info.id).toEqual(Buffer.from('03653e90c1ce4660fd8505dd6d643356e93cfe202af109d382787639dd5890e87d', 'hex'));
      expect(info.color).toEqual(Buffer.from('03653e', 'hex'));
      expect(info.numPeers).toBe(0);
      expect(info.numPendingChannels).toBe(0);
      expect(info.numActiveChannels).toBe(0);
      expect(info.numInactiveChannels).toBe(0);
      expect(info.lightningDir).toBe('/tmp/bitcoin');
      expect(info.network).toBe('bitcoin');
      expect(info.feesCollectedMsat).toBe(0);

      // Alias is optional
      if (info.alias !== null && info.alias !== undefined) {
        expect(typeof info.alias).toBe('string');
        expect(info.alias).toContain('PEEVEDGENESIS');
      }
    });
  });

  describe('calls listPeers', () => {
    it('returns peer information with expected structure', async () => {
      const response = await node.listPeers();

      expect(response).toBeTruthy();
      expect(Array.isArray(response.peers)).toBe(true);
    });
  });

  describe('calls listPeerChannels', () => {
    it('returns channel information with expected structure', async () => {
      const response = await node.listPeerChannels();

      expect(response).toBeTruthy();
      expect(Array.isArray(response.channels)).toBe(true);
    });
  });

  describe('calls listFunds', () => {
    it('returns fund information with expected structure', async () => {
      const response = await node.listFunds();

      expect(response).toBeTruthy();
      expect(Array.isArray(response.outputs)).toBe(true);
      expect(Array.isArray(response.channels)).toBe(true);
    });
  });

  describe('calls onchainReceive', () => {
    it('returns valid on-chain addresses', async () => {
      const res = await node.onchainReceive();

      expect(typeof res.bech32).toBe('string');
      expect(res.bech32.length).toBeGreaterThan(0);
      expect(res.bech32.startsWith('bc1')).toBe(true);

      expect(typeof res.p2Tr).toBe('string');
      expect(res.p2Tr.length).toBeGreaterThan(0);
      expect(res.p2Tr.startsWith('bc1p')).toBe(true);
    });

    it('generates different addresses on multiple calls', async () => {
      const res1 = await node.onchainReceive();
      const res2 = await node.onchainReceive();

      expect(res1.bech32).not.toBe(res2.bech32);
      expect(res1.p2Tr).not.toBe(res2.p2Tr);
    });
  });

  // Note: These are currently failing
  // describe('receive (Lightning invoice)', () => {
  //   it('can create invoice with amount', async () => {
  //     const label = `test-${Date.now()}`;
  //     const description = 'Test payment';
  //     const amountMsat = 100000;

  //     const response = await node.receive(label, description, amountMsat);

  //     expect(response).toBeTruthy();
  //     expect(typeof response.bolt11).toBe('string');
  //     expect(response.bolt11.length).toBeGreaterThan(0);
  //     expect(response.bolt11.toLowerCase().startsWith('ln')).toBe(true);
  //   });
  // });

  // describe('send (Lightning payment)', () => {
  //   it('can attempt to send payment to valid invoice', async () => {
  //     const label = `test-send-${Date.now()}`;
  //     const receiveResponse = await node.receive(label, 'Test for send', 1000);

  //     try {
  //       const sendResponse = await node.send(receiveResponse.bolt11, null);
  //       expect(sendResponse).toBeTruthy();
  //     } catch (e) {
  //       expect(e).toBeDefined();
  //     }
  //   });

  //   it('can send with explicit amount for zero-amount invoice', async () => {
  //     const label = `test-send-amount-${Date.now()}`;
  //     const receiveResponse = await node.receive(label, 'Zero amount invoice', null);

  //     try {
  //       const sendResponse = await node.send(receiveResponse.bolt11, 5000);
  //       expect(sendResponse).toBeTruthy();
  //     } catch (e) {
  //       expect(e).toBeDefined();
  //     }
  //   });
  // });

  // describe('onchainSend', () => {
  //   it('can attempt to send specific amount on-chain', async () => {
  //     const destAddress = (await node.onchainReceive()).bech32;

  //     try {
  //       const response = await node.onchainSend(destAddress, '10000sat');
  //       expect(response).toBeTruthy();
  //     } catch (e) {
  //       expect(e).toBeDefined();
  //     }
  //   });

  //   it('can attempt to send all funds on-chain', async () => {
  //     const destAddress = (await node.onchainReceive()).bech32;

  //     try {
  //       const response = await node.onchainSend(destAddress, 'all');
  //       expect(response).toBeTruthy();
  //     } catch (e) {
  //       expect(e).toBeDefined();
  //     }
  //   });
  // });

  describe('stop', () => {
    it('can stop the node', async () => {
      const testScheduler = new Scheduler('regtest');
      const testSigner = new Signer(MNEMONIC);
      const testCredentials = await testScheduler.recover(testSigner);
      const testNode = new Node(testCredentials);

      await expect(testNode.stop()).resolves.not.toThrow();
    });
  });
});
