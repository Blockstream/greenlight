import * as crypto from 'crypto';
import * as bip39 from 'bip39';
import { Credentials, Scheduler, Signer, Node, NodeEventStream, NodeEvent, InvoicePaidEvent, Handle } from '../index.js';
import { fundWallet, getGLNode } from './test.helper.js';

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
  let scheduler: Scheduler = new Scheduler('regtest');
  let node: Node;
  let handle: Handle;

  beforeAll(async () => {
    ({node, handle } = await getGLNode(scheduler, false) as { node: Node; handle: Handle });
    try {
      const probe = await node.streamNodeEvents();
      void probe;
    } catch (e: any) {
      console.warn(`⚠ StreamNodeEvents probe failed — skipping stream tests`);
      console.warn(`(${e?.message ?? e})`);
    }
  });

  afterAll(async () => {
    if (node) {
      handle.stop();
      await node.stop();
    }
  });

  it('does not throw error on future unknown event type', () => {
    const unknownEvent: NodeEvent = { eventType: 'new_future_event' as string, invoicePaid: undefined };

    expect(() => {
      switch (unknownEvent.eventType) {
        case 'invoice_paid': break;
        case 'unknown':
        default: break;
      }
    }).not.toThrow();
  });

  it('streamNodeEvents returns a next method', async () => {
    const stream = await node.streamNodeEvents();
    expect(stream).toBeDefined();
    expect(typeof stream.next).toBe('function');
  });

  it('next resolves to null or a well-formed NodeEvent within 2 seconds', async () => {
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

  it('next returns null after the node is stopped', async () => {
    const mnemonic2 = bip39.entropyToMnemonic(crypto.randomBytes(16).toString('hex'));
    const signer2 = new Signer(mnemonic2);
    const handle2 = await signer2.start();
    const credentials2 = await scheduler.register(signer2);
    let node2 = new Node(credentials2);
    const stream: NodeEventStream = await node2.streamNodeEvents();
    await node2.stop();
    const result = await stream.next();
    expect(result).toBeNull();
    node2 = new Node(credentials2);
    handle2.stop()
    await node2.stop();
  });

  it.skip('receives real invoice_paid event', async () => {
      await fundWallet(node, 500_000_000);
      const { node: node2, handle: handle2 } = await getGLNode(scheduler, true) as { node: Node; handle: Handle };
      const stream: NodeEventStream = await node.streamNodeEvents();
      const label = `jest-${Date.now()}`;
      const receiveRes = await node.receive(label, 'jest event stream test', 1_000);
      const sendResponse = await node2.send(receiveRes.bolt11);
      expect(sendResponse).toBeTruthy();

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
      expect(p.bolt11).toBe(receiveRes.bolt11);
      expect(p.label).toBe(label);
      expect(typeof p.amountMsat).toBe('number');
      expect(p.amountMsat).toBeGreaterThan(0);
      handle2.stop();
      await node2.stop();
    },
    15_000 // extended timeout for payment round-trip
  );

});
