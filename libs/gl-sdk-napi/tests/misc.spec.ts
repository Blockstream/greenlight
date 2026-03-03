import * as crypto from 'crypto';
import * as bip39 from 'bip39';
import { Credentials, Scheduler, Signer, Node } from '../index.js';

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
    const rand: Buffer = crypto.randomBytes(16);
    const MNEMONIC: string = bip39.entropyToMnemonic(rand.toString("hex"));
    const signer = new Signer(MNEMONIC);
    expect(signer).toBeTruthy();
  });

  it('can return a node id', async () => {
    const rand: Buffer = crypto.randomBytes(16);
    const MNEMONIC: string = bip39.entropyToMnemonic(rand.toString("hex"));
    const signer = new Signer(MNEMONIC);
    const nodeId = signer.nodeId();

    expect(Buffer.isBuffer(nodeId)).toBe(true);
    expect(nodeId.length).toBeGreaterThan(0);
  });

  it('returns consistent node id for same mnemonic', async () => {
    const rand: Buffer = crypto.randomBytes(16);
    const MNEMONIC: string = bip39.entropyToMnemonic(rand.toString("hex"));
    const signer1 = new Signer(MNEMONIC);
    const signer2 = new Signer(MNEMONIC);

    const nodeId1 = signer1.nodeId();
    const nodeId2 = signer2.nodeId();

    expect(nodeId1.equals(nodeId2)).toBe(true);
  });

  it('can be constructed with different mnemonics', async () => {
    const rand2: Buffer = crypto.randomBytes(16);
    const MNEMONIC2: string = bip39.entropyToMnemonic(rand2.toString("hex"));
    const signer = new Signer(MNEMONIC2);
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
  let credentials: Credentials;
  let node: Node;

  beforeAll(async () => {
    const rand: Buffer = crypto.randomBytes(16);
    const MNEMONIC: string = bip39.entropyToMnemonic(rand.toString("hex"));
    scheduler = new Scheduler('regtest');
    signer = new Signer(MNEMONIC);
    credentials = await scheduler.register(signer);
    node = new Node(credentials);
  });

  it('can recover credentials', async () => {
    if (node) { await node.stop(); }
    const recovered = await scheduler.recover(signer);
    expect(recovered).toBeInstanceOf(Credentials);
    expect((await recovered.save()).length).toBeGreaterThan(0);
  });

  it('handles registration of existing node (falls back to recovery)', async () => {
    try {
      if (node) { await node.stop(); }
      // Trying to register the same signer again should throw an error, which we catch to then test recovery
      const credentials2 = await scheduler.register(signer);
      expect(credentials2).toBeInstanceOf(Credentials);
    } catch (e) {
      const recovered = await scheduler.recover(signer);
      expect(recovered).toBeInstanceOf(Credentials);
    }
  });
});
