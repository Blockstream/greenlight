import { parseInput } from '../index.js';

const VALID_NODE_ID =
  '02eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619';

const BOLT11_INVOICE =
  'lnbc110n1p38q3gtpp5ypz09jrd8p993snjwnm68cph4ftwp22le34xd4r8ftspwshxhm' +
  'nsdqqxqyjw5qcqpxsp5htlg8ydpywvsa7h3u4hdn77ehs4z4e844em0apjyvmqfkzqhh' +
  'd2q9qgsqqqyssqszpxzxt9uuqzymr7zxcdccj5g69s8q7zzjs7sgxn9ejhnvdh6gqjcy' +
  '22mss2yexunagm5r2gqczh8k24cwrqml3njskm548aruhpwssq9nvrvz';

describe('parseInput', () => {
  describe('BOLT11 invoices (no HTTP)', () => {
    it('classifies a valid BOLT11 invoice', async () => {
      const result = await parseInput(BOLT11_INVOICE);
      expect(result.type).toBe('bolt11');
      expect(result.bolt11).toBeDefined();
      expect(result.bolt11!.bolt11).toBe(BOLT11_INVOICE);
    });

    it('strips a lowercase lightning: prefix', async () => {
      const result = await parseInput(`lightning:${BOLT11_INVOICE}`);
      expect(result.type).toBe('bolt11');
    });

    it('strips an uppercase LIGHTNING: prefix', async () => {
      const result = await parseInput(`LIGHTNING:${BOLT11_INVOICE}`);
      expect(result.type).toBe('bolt11');
    });
  });

  describe('node IDs (no HTTP)', () => {
    it('classifies a valid compressed pubkey', async () => {
      const result = await parseInput(VALID_NODE_ID);
      expect(result.type).toBe('node_id');
      expect(result.nodeId).toBe(VALID_NODE_ID);
    });

    it('rejects a 66-char string that is not valid hex', async () => {
      await expect(
        parseInput('not_valid_hex_at_all_but_66_chars_long_xxxxxxxxxxxxxxxxxxxxxxxxxxx'),
      ).rejects.toThrow();
    });

    it('rejects an uncompressed (0x04) pubkey', async () => {
      await expect(
        parseInput('04eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619'),
      ).rejects.toThrow();
    });
  });

  describe('error cases that resolve before HTTP', () => {
    it('rejects empty input', async () => {
      await expect(parseInput('')).rejects.toThrow();
    });

    it('rejects whitespace-only input', async () => {
      await expect(parseInput('   ')).rejects.toThrow();
    });

    it('rejects unrecognized garbage', async () => {
      await expect(parseInput('hello world')).rejects.toThrow();
    });

    it('rejects an invalid LNURL bech32 string before HTTP', async () => {
      await expect(parseInput('LNURL1INVALIDDATA')).rejects.toThrow();
    });

    it('rejects a malformed Lightning Address (no dot in domain)', async () => {
      await expect(parseInput('user@localhost')).rejects.toThrow();
    });

    it('rejects an empty local-part Lightning Address', async () => {
      await expect(parseInput('@example.com')).rejects.toThrow();
    });

    it('rejects an empty domain Lightning Address', async () => {
      await expect(parseInput('user@')).rejects.toThrow();
    });
  });
});
