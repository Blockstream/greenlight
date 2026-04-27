import * as crypto from 'crypto';
import * as bip39 from 'bip39';
import { Config, register } from '../index.js';

describe('Greenlight node', () => {
  it('can be set up via register()', async () => {
    const rand: Buffer = crypto.randomBytes(16);
    const MNEMONIC: string = bip39.entropyToMnemonic(rand.toString('hex'));
    const config = new Config().withNetwork('regtest');
    const node = await register(MNEMONIC, undefined, config);
    expect(node).toBeTruthy();
    node.disconnect();
    await node.stop();
  });
});
