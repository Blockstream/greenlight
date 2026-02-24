import { describe, it, expect, beforeAll } from '@jest/globals';
import { Credentials, Scheduler, Signer } from '../index.js';

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
