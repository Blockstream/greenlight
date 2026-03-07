import * as crypto from 'crypto';
import * as bip39 from 'bip39';
import { Credentials, Scheduler, Signer, Node } from '../index.js';
import { fundWallet, lspInfo } from './test.helper';

describe('Node', () => {
  let node: Node;
  let credentials: Credentials;

  beforeEach(async () => {
    const rand: Buffer = crypto.randomBytes(16);
    const MNEMONIC: string = bip39.entropyToMnemonic(rand.toString("hex"));
    const scheduler = new Scheduler('regtest');
    const signer = new Signer(MNEMONIC);
    credentials = await scheduler.register(signer);
    node = new Node(credentials);
  });

  afterEach(async () => {
    if (node) {
      await node.stop();
    }
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
    it.skip('can attempt to send specific amount on-chain', async () => {
      await fundWallet(node, 500_000_000);
      const rand2: Buffer = crypto.randomBytes(16);
      const MNEMONIC2: string = bip39.entropyToMnemonic(rand2.toString("hex"));
      const scheduler2 = new Scheduler('regtest');
      const signer2 = new Signer(MNEMONIC2);
      const credentials2 = await scheduler2.register(signer2);
      const node2 = new Node(credentials2);
      const destAddress = (await node2.onchainReceive()).bech32;
      const response = await node.onchainSend(destAddress, '10000sat');
      expect(response).toBeTruthy();
    });

    it.skip('can attempt to send all funds on-chain', async () => {
      await fundWallet(node, 500_000_000);
      const rand2: Buffer = crypto.randomBytes(16);
      const MNEMONIC2: string = bip39.entropyToMnemonic(rand2.toString("hex"));
      const scheduler2 = new Scheduler('regtest');
      const signer2 = new Signer(MNEMONIC2);
      const credentials2 = await scheduler2.register(signer2);
      const node2 = new Node(credentials2);
      const destAddress = (await node2.onchainReceive()).bech32;
      const response = await node.onchainSend(destAddress, 'all');
      expect(response).toBeTruthy();
    });
  });

  describe('calls receive', () => {
    it.skip('can create invoice with amount', async () => {
      const { rpcSocket, nodeId } = lspInfo();
      // Connect to the LSP as a peer
      console.log('LSP Node Info:', rpcSocket, nodeId);
      // await node.connectPeer(lspNodeInfo.id, lspNodeInfo.bindings[0].address, lspNodeInfo.bindings[0].port);
      // await new Promise(resolve => setTimeout(resolve, 2000));

      const label = `test-${Date.now()}`;
      const description = 'Test payment';
      const amountMsat = 100000;
      const response = await node.receive(label, description, amountMsat);
      expect(response).toBeTruthy();
      expect(typeof response.bolt11).toBe('string');
      expect(response.bolt11.length).toBeGreaterThan(0);
      expect(response.bolt11.toLowerCase().startsWith('ln')).toBe(true);
    });
  });

  describe('calls send', () => {
    it.skip('can attempt to send payment to valid invoice', async () => {
      const { rpcSocket, nodeId } = lspInfo();
      await fundWallet(node, 500_000_000);
      const rand2: Buffer = crypto.randomBytes(16);
      const MNEMONIC2: string = bip39.entropyToMnemonic(rand2.toString("hex"));
      const scheduler2 = new Scheduler('regtest');
      const signer2 = new Signer(MNEMONIC2);
      const credentials2 = await scheduler2.register(signer2);
      const node2 = new Node(credentials2);
      const receiveRes = await node2.receive(`test-${Date.now()}`, 'Test payment', 100000);
      const sendResponse = await node.send(receiveRes.bolt11);
      expect(sendResponse).toBeTruthy();
    });

    it.skip('can send with explicit amount for zero-amount invoice', async () => {
      const { rpcSocket, nodeId } = lspInfo();
      await fundWallet(node, 500_000_000);
      const rand2: Buffer = crypto.randomBytes(16);
      const MNEMONIC2: string = bip39.entropyToMnemonic(rand2.toString("hex"));
      const scheduler2 = new Scheduler('regtest');
      const signer2 = new Signer(MNEMONIC2);
      const credentials2 = await scheduler2.register(signer2);
      const node2 = new Node(credentials2);
      const receiveRes = await node2.receive(`test-${Date.now()}`, 'Test payment');
      const sendResponse = await node.send(receiveRes.bolt11);
      expect(sendResponse).toBeTruthy();
    });
  });

  describe('calls stop', () => {
    it('can stop the node', async () => {
      const rand: Buffer = crypto.randomBytes(16);
      const MNEMONIC: string = bip39.entropyToMnemonic(rand.toString("hex"));
      const testScheduler = new Scheduler('regtest');
      const testSigner = new Signer(MNEMONIC);
      const testCredentials = await testScheduler.register(testSigner);
      const testNode = new Node(testCredentials);

      await expect(testNode.stop()).resolves.not.toThrow();
    });
  });
});
