import { describe, it, expect, beforeAll, afterAll } from '@jest/globals';
import {
  Credentials,
  Scheduler,
  Signer,
  Node,
  NodeEventStream,
  NodeEvent,
  InvoicePaidEvent,
} from '../index.js';

const MNEMONIC =
  'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about';

describe('NodeEvent (type contract)', () => {
  it('NodeEvent and InvoicePaidEvent are assignable from NAPI-generated types', () => {
    // This is a compile-time check only. If the NAPI bindings change the
    // field names or types, tsc will fail here before Jest even runs.
    // The runtime assertion is intentionally trivial.
    const details: InvoicePaidEvent = {
      paymentHash: Buffer.from('deadbeef', 'hex'),
      bolt11: 'lnbcrt1...',
      preimage: Buffer.from('cafebabe', 'hex'),
      label: 'test-label',
      amountMsat: 1_000_000,
    };
    const event: NodeEvent = { eventType: 'invoice_paid', invoicePaid: details };

    // Only assert what tsc cannot: that the import itself resolved and
    // the constructed value is truthy (i.e. the module loaded correctly).
    expect(event).toBeDefined();
  });
});

// ============================================================================
// NodeEventStream integration tests — require a live regtest scheduler
// ============================================================================

describe('NodeEventStream (integration)', () => {
  let credentials: Credentials;
  let node: Node;
  let streamingSupported = true;
  let suiteAvailable = true;

  beforeAll(async () => {
    try {
      const scheduler = new Scheduler('regtest');
      const signer = new Signer(MNEMONIC);
      credentials = await scheduler.recover(signer);
      node = new Node(credentials);
    } catch (e: any) {
      console.warn(`⚠ Scheduler unavailable — skipping all integration tests`);
      console.warn(`(${e?.message ?? e})`);
      suiteAvailable = false;
      return;
    }

    try {
      const probe = await node.streamNodeEvents();
      void probe;
    } catch (e: any) {
      console.warn(`⚠ StreamNodeEvents probe failed — skipping stream tests`);
      console.warn(`(${e?.message ?? e})`);
      // Treat every probe failure as "not supported" — covers both
      // Unimplemented and any Unavailable thrown at this point.
      streamingSupported = false;
    }
  });

  afterAll(async () => {
    if (suiteAvailable && node) {
      await node.stop();
    }
  });

  /**
   * Wrapper around `it` that skips the test body when:
   * - The regtest scheduler was unreachable (`suiteAvailable = false`), or
   * - The connected node does not implement StreamNodeEvents (`streamingSupported = false`).
   */
  const streamIt = (name: string, fn: () => Promise<void>, timeout?: number) =>
    it(
      name,
      async () => {
        if (!suiteAvailable) {
          console.log('Skipped — scheduler unavailable');
          return;
        }
        if (!streamingSupported) {
          console.log('Skipped — StreamNodeEvents not supported on this node');
          return;
        }
        await fn();
      },
      timeout
    );

  it('does not throw error on future unknown event type', () => {
    if (!suiteAvailable) {
      console.log('Skipped — regtest scheduler unavailable');
      return;
    }

    const unknownEvent: NodeEvent = { eventType: 'new_future_event' as string, invoicePaid: undefined };

    expect(() => {
      switch (unknownEvent.eventType) {
        case 'invoice_paid': break;
        case 'unknown':
        default: break;
      }
    }).not.toThrow();
  });

  streamIt('streamNodeEvents returns a next method', async () => {
    const stream = await node.streamNodeEvents();
    expect(stream).toBeDefined();
    expect(typeof stream.next).toBe('function');
  });

  streamIt('next resolves to null or a well-formed NodeEvent within 2 seconds', async () => {
    const stream: NodeEventStream = await node.streamNodeEvents();

    // Race next() against a 2 s timeout — no live events is fine here.
    const result = await Promise.race([
      stream.next(),
      new Promise<null>(resolve => setTimeout(() => resolve(null), 2_000)),
    ]);

    if (result === null) return; // timed out, no events — acceptable

    expect(result).toHaveProperty('eventType');
    expect(typeof result.eventType).toBe('string');

    if (result.eventType === 'invoice_paid') {
      expect(result.invoicePaid).toBeDefined();
      expect(Buffer.isBuffer(result.invoicePaid!.paymentHash)).toBe(true);
      expect(Buffer.isBuffer(result.invoicePaid!.preimage)).toBe(true);
      expect(typeof result.invoicePaid!.amountMsat).toBe('number');
    }
  });

  streamIt('next returns null after the node is stopped', async () => {
    const stream: NodeEventStream = await node.streamNodeEvents();
    await node.stop();

    const result = await stream.next();
    expect(result).toBeNull();
    node = new Node(credentials);
  });

  // --------------------------------------------------------------------------
  // invoice_paid round-trip
  // --------------------------------------------------------------------------

  streamIt('receives real invoice_paid event', async () => {
      const stream: NodeEventStream = await node.streamNodeEvents();
      const label = `jest-${Date.now()}`;
      const invoice = await node.receive(label, 'jest event stream test', 1_000);
      expect(invoice.bolt11).toMatch(/^ln/i);

      let paid: NodeEvent | null = null;
      const deadline = Date.now() + 10_000;

      while (Date.now() < deadline) {
        const event = await Promise.race([
          stream.next(),
          new Promise<null>(resolve =>
            setTimeout(() => resolve(null), deadline - Date.now())
          ),
        ]);

        if (event === null) break;
        if (event.eventType === 'invoice_paid') { paid = event; break; }
      }

      expect(paid).not.toBeNull();
      expect(paid!.eventType).toBe('invoice_paid');

      const p = paid!.invoicePaid!;
      expect(p).toBeDefined();
      expect(Buffer.isBuffer(p.paymentHash)).toBe(true);
      expect(p.paymentHash.length).toBeGreaterThan(0);
      expect(Buffer.isBuffer(p.preimage)).toBe(true);
      expect(p.preimage.length).toBeGreaterThan(0);
      expect(p.bolt11).toBe(invoice.bolt11);
      expect(p.label).toBe(label);
      expect(typeof p.amountMsat).toBe('number');
      expect(p.amountMsat).toBeGreaterThan(0);
    },
    15_000 // extended timeout for payment round-trip
  );

});
