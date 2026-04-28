import { parseInput } from '../index.js';

const VALID_NODE_ID =
  '02eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619';

const BOLT11_INVOICE =
  'lnbc110n1p38q3gtpp5ypz09jrd8p993snjwnm68cph4ftwp22le34xd4r8ftspwshxhm' +
  'nsdqqxqyjw5qcqpxsp5htlg8ydpywvsa7h3u4hdn77ehs4z4e844em0apjyvmqfkzqhh' +
  'd2q9qgsqqqyssqszpxzxt9uuqzymr7zxcdccj5g69s8q7zzjs7sgxn9ejhnvdh6gqjcy' +
  '22mss2yexunagm5r2gqczh8k24cwrqml3njskm548aruhpwssq9nvrvz';

const VALID_LNURL_BECH32 =
  'LNURL1DP68GURN8GHJ7CMFWP5X2UNSW4HXKTNRDAKJ7CTSDYHHVVF0D3H82UNV9UCSAXQZE2';

describe('parseInput (synchronous, offline)', () => {
  describe('BOLT11 invoices', () => {
    it('classifies a valid BOLT11 invoice', () => {
      const result = parseInput(BOLT11_INVOICE);
      expect(result.type).toBe('bolt11');
      expect(result.bolt11).toBeDefined();
      expect(result.bolt11!.bolt11).toBe(BOLT11_INVOICE);
    });

    it('strips a lowercase lightning: prefix', () => {
      const result = parseInput(`lightning:${BOLT11_INVOICE}`);
      expect(result.type).toBe('bolt11');
    });

    it('strips an uppercase LIGHTNING: prefix', () => {
      const result = parseInput(`LIGHTNING:${BOLT11_INVOICE}`);
      expect(result.type).toBe('bolt11');
    });
  });

  describe('node IDs', () => {
    it('classifies a valid compressed pubkey', () => {
      const result = parseInput(VALID_NODE_ID);
      expect(result.type).toBe('node_id');
      expect(result.nodeId).toBe(VALID_NODE_ID);
    });

    it('rejects a 66-char string that is not valid hex', () => {
      expect(() =>
        parseInput('not_valid_hex_at_all_but_66_chars_long_xxxxxxxxxxxxxxxxxxxxxxxxxxx'),
      ).toThrow();
    });

    it('rejects an uncompressed (0x04) pubkey', () => {
      expect(() =>
        parseInput('04eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619'),
      ).toThrow();
    });
  });

  describe('LNURL bech32 / Lightning Address', () => {
    it('decodes an LNURL bech32 to its underlying URL', () => {
      const result = parseInput(VALID_LNURL_BECH32);
      expect(result.type).toBe('lnurl');
      expect(result.lnurl).toBeDefined();
      expect(result.lnurl!.startsWith('https://')).toBe(true);
    });

    it('returns a Lightning Address as the user@host form', () => {
      const result = parseInput('user@example.com');
      expect(result.type).toBe('lnurl_address');
      expect(result.lnurlAddress).toBe('user@example.com');
    });
  });

  describe('error cases', () => {
    it('rejects empty input', () => {
      expect(() => parseInput('')).toThrow();
    });

    it('rejects whitespace-only input', () => {
      expect(() => parseInput('   ')).toThrow();
    });

    it('rejects unrecognized garbage', () => {
      expect(() => parseInput('hello world')).toThrow();
    });

    it('rejects an invalid LNURL bech32 string', () => {
      expect(() => parseInput('LNURL1INVALIDDATA')).toThrow();
    });

    it('rejects a malformed Lightning Address (no dot in domain)', () => {
      expect(() => parseInput('user@localhost')).toThrow();
    });

    it('rejects an empty local-part Lightning Address', () => {
      expect(() => parseInput('@example.com')).toThrow();
    });

    it('rejects an empty domain Lightning Address', () => {
      expect(() => parseInput('user@')).toThrow();
    });
  });
});
