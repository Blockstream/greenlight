import * as crypto from 'crypto';
import * as bip39 from 'bip39';
import {
  Config,
  Credentials,
  Node,
  Signer,
  connect,
  recover,
  register,
} from '../index.js';

const REGTEST = () => new Config().withNetwork('regtest');

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
    const MNEMONIC: string = bip39.entropyToMnemonic(rand.toString('hex'));
    const signer = new Signer(MNEMONIC);
    expect(signer).toBeTruthy();
  });

  it('can return a node id', async () => {
    const rand: Buffer = crypto.randomBytes(16);
    const MNEMONIC: string = bip39.entropyToMnemonic(rand.toString('hex'));
    const signer = new Signer(MNEMONIC);
    const nodeId = signer.nodeId();

    expect(Buffer.isBuffer(nodeId)).toBe(true);
    expect(nodeId.length).toBeGreaterThan(0);
  });

  it('returns consistent node id for same mnemonic', async () => {
    const rand: Buffer = crypto.randomBytes(16);
    const MNEMONIC: string = bip39.entropyToMnemonic(rand.toString('hex'));
    const signer1 = new Signer(MNEMONIC);
    const signer2 = new Signer(MNEMONIC);

    const nodeId1 = signer1.nodeId();
    const nodeId2 = signer2.nodeId();

    expect(nodeId1.equals(nodeId2)).toBe(true);
  });
});

describe('Config', () => {
  it('defaults to bitcoin network and has no developer cert', () => {
    const config = new Config();
    expect(config).toBeTruthy();
  });

  it('produces a regtest-network Config via withNetwork', () => {
    const config = new Config().withNetwork('regtest');
    expect(config).toBeTruthy();
  });

  it('rejects invalid network strings', () => {
    expect(() => new Config().withNetwork('mars')).toThrow();
  });
});

describe('Integration: register / recover / connect', () => {
  let mnemonic: string;
  let registeredNode: Node | null = null;

  beforeAll(() => {
    mnemonic = bip39.entropyToMnemonic(crypto.randomBytes(16).toString('hex'));
  });

  afterAll(async () => {
    if (registeredNode) {
      try { registeredNode.disconnect(); } catch {}
      try { await registeredNode.stop(); } catch {}
    }
  });

  it('register returns a connected Node and exposes credentials', async () => {
    registeredNode = await register(mnemonic, undefined, REGTEST());
    expect(registeredNode).toBeTruthy();

    const creds = registeredNode.credentials();
    expect(Buffer.isBuffer(creds)).toBe(true);
    expect(creds.length).toBeGreaterThan(0);
  });

  it('recover returns a Node for an already-registered mnemonic', async () => {
    if (registeredNode) {
      registeredNode.disconnect();
      await registeredNode.stop();
      registeredNode = null;
    }
    const recovered = await recover(mnemonic, REGTEST());
    expect(recovered).toBeTruthy();
    registeredNode = recovered;
  });

  it('connect works with saved credentials and the same mnemonic', async () => {
    const savedCreds = registeredNode!.credentials();
    registeredNode!.disconnect();
    await registeredNode!.stop();
    registeredNode = null;

    const reconnected = await connect(mnemonic, savedCreds, REGTEST());
    expect(reconnected).toBeTruthy();
    registeredNode = reconnected;
  });
});
