import { Node, Scheduler } from '../index.js';
import { getGLNode, fundWallet, getLspInvoice, SdkNode } from './test.helper';

describe('Node', () => {
  let scheduler: Scheduler = new Scheduler('regtest');
  let glNodes: SdkNode[] = [];
  let node: Node;

  beforeEach(async () => {
    glNodes.push(await getGLNode(scheduler, false));
    node = glNodes[0].node;
  });

  afterEach(async () => {
    for (const { node: n } of glNodes) {
      try { n.disconnect(); } catch {}
      try { await n.stop(); } catch {}
    }
    glNodes = [];
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
      expect(info.numPeers).toBe(0);
      expect(info.numPendingChannels).toBe(0);
      expect(info.numActiveChannels).toBe(0);
      expect(info.numInactiveChannels).toBe(0);
      expect(info.lightningDir).toContain('/tmp/');
      expect(info.network).toBe('regtest');
      expect(info.feesCollectedMsat).toBe(0);

      // Alias is optional
      if (info.alias !== null && info.alias !== undefined) {
        expect(typeof info.alias).toBe('string');
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
      expect(res.bech32.startsWith('bcrt1')).toBe(true);

      expect(typeof res.p2Tr).toBe('string');
      expect(res.p2Tr.length).toBeGreaterThan(0);
      expect(res.p2Tr.startsWith('bcrt1p')).toBe(true);
    });

    it('generates different addresses on multiple calls', async () => {
      const res1 = await node.onchainReceive();
      const res2 = await node.onchainReceive();

      expect(res1.bech32).not.toBe(res2.bech32);
      expect(res1.p2Tr).not.toBe(res2.p2Tr);
    });
  });

  describe('calls onchainSend', () => {
    it('can send specific amount on-chain', async () => {
      await fundWallet(node, 500_000_000);
      const extraGLNode = await getGLNode(scheduler, true);
      glNodes.push(extraGLNode);
      const destAddress = (await extraGLNode.node.onchainReceive()).bech32;
      const response = await node.onchainSend(destAddress, '10000sat');
      expect(response).toBeTruthy();
    });

    it('can attempt to send all funds on-chain', async () => {
      await fundWallet(node, 500_000_000);
      const extraGLNode = await getGLNode(scheduler, true);
      glNodes.push(extraGLNode);
      const destAddress = (await extraGLNode.node.onchainReceive()).bech32;
      const response = await node.onchainSend(destAddress, 'all');
      expect(response).toBeTruthy();
    });
  });

  describe('calls receive', () => {
    it('can create invoice with amount', async () => {
      const extraGLNode = await getGLNode(scheduler, true);
      glNodes.push(extraGLNode);
      const label = `test-${Date.now()}`;
      const description = 'Test payment';
      const amountMsat = 100000;
      const response = await extraGLNode.node.receive(label, description, amountMsat);
      expect(response).toBeTruthy();
      expect(typeof response.bolt11).toBe('string');
      expect(response.bolt11.length).toBeGreaterThan(0);
      expect(response.bolt11.toLowerCase().startsWith('ln')).toBe(true);
    });
  });

  describe('calls send', () => {
    it.skip('can attempt to send payment to valid invoice', async () => {
      await fundWallet(node, 500_000_000);
      const bolt11 = await getLspInvoice(100_000);
      const sendResponse = await node.send(bolt11);
      expect(sendResponse).toBeTruthy();
    });

    it.skip('can send with explicit amount for zero-amount invoice', async () => {
      await fundWallet(node, 500_000_000);
      const bolt11 = await getLspInvoice(0);
      const sendResponse = await node.send(bolt11, 200_000);
      expect(sendResponse).toBeTruthy();
    });
  });
});
