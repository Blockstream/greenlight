# Signer State

The signer is built on the [Validating Lightning Signer][vls] (VLS).
VLS does not sign blindly: before signing it checks the request against
what it knows about the node, for example the current state of each
channel, which destinations the on-chain wallet may send to, and which
invoices and preimages it has seen. This knowledge is the _signer
state_.

Greenlight signers are stateless by design. They do not own durable
storage for the signer state, which is what allows a signer to run on
a phone, in a short-lived process, or on several devices at once.
Instead, the Greenlight node stores the signer state and attaches it to
every request it sends to the signer.

This creates a tension with the [security model](security.md): the
signer protects funds against an attacker with access to the node
infrastructure, yet that same infrastructure stores the state the
signer relies on. This page describes how the signer can use state it
receives from an untrusted store, what is protected today, what is
planned, and which options an application embedding the signer
controls.

## Contents of the signer state

The signer state is a key-value map. Every entry consists of:

| Field | Description |
|-------|-------------|
| `key` | The entry name, prefixed by its class: `nodes/`, `nodestates/`, `channels/`, `allowlists/`, `trackers/`, `peers/` |
| `version` | A counter that increases with every change to this key |
| `value` | The serialized VLS object |
| `signature` | The signer's signature over the key, version and value |

Deleting a key leaves a _tombstone_ behind: the entry is kept with the
highest possible version and an empty value, so that an older value of
the key cannot be merged back in.

## Synchronization

The node and the signer keep their copies of the state in sync by
exchanging only what changed:

 1. When a signer connects, the node sends it a heartbeat request that
    carries a full snapshot of the state.
 2. Every following request carries only the entries that changed
    since the node last sent state to that signer.
 3. The signer merges incoming entries into its in-memory copy. New
    keys are added, higher versions replace lower ones, lower versions
    are ignored.
 4. The signer processes the request and returns the entries it
    changed, signed.
 5. The node merges and persists the returned entries _before_ it
    hands the signer's response on. If the entries cannot be stored,
    the response is dropped, so the node never acts on a signature whose
    state was lost.

The node pipelines requests, so several can be in flight at once, each
carrying the state as it was when the request was sent. The signer
handles them one after the other, and its in-memory state is always at
least as new as the state attached to a request, so this is safe.

### Multiple signers

A node can have several signers connected at the same time, for example
the same wallet on a phone and a desktop. Every connected signer
receives every request, the first response wins, and the node merges
the state returned by all of them key by key. Because concurrent
requests touch different keys, all signers make progress, and the node
never needs to create or modify a signature itself.

Some cryptographic operations are not fully determined by their inputs.
Two signers handling the same request may therefore return different
but equally valid entries. The signer accepts either.

## Protecting the state

There are three properties the signer needs from the state it receives:

| Property | Question | Mechanism | Status |
|----------|----------|-----------|--------|
| Authenticity | Was every entry written by a signer of this node? | State signatures | Available |
| Freshness | Is any entry older than one this signer has already seen? | State anchor | Planned |
| Completeness | Is any entry this signer has seen missing? | State anchor | Planned |

### Authenticity: state signatures

Each entry is signed by a _state signing key_ that is derived from the
node secret. All signers of the same node derive the same key, so they
can verify each other's entries, while the node, which never has the
secret, cannot produce valid signatures.

How strictly the signer verifies these signatures is controlled by the
_state signature mode_:

| Mode | Missing signature | Invalid signature | Intended use |
|------|-------------------|-------------------|--------------|
| `off` | Accepted | Accepted | Development and tests only |
| `soft` | Accepted, and signed on the way out | Refused | Migrating nodes whose state predates state signatures |
| `hard` | Refused | Refused | Production, once all state is signed |

`soft` is the current default. In `soft` mode a signer signs unsigned
entries it receives, which gradually migrates nodes created before
state signing existed. Only `hard` mode guarantees that every entry the
signer acts on was written by a signer of this node.

!!! note "Planned default"
	The default will move from `soft` to `hard` once the fleet's state
	is fully signed. Applications embedding the signer should plan for
	running in `hard` mode.

The mode is part of the signer configuration:

=== "Rust"

	```rust
	use gl_client::signer::{Signer, SignerConfig, StateSignatureMode};

	let signer = Signer::new_with_config(
	    seed,
	    network,
	    creds,
	    SignerConfig {
	        state_signature_mode: StateSignatureMode::Hard,
	        ..Default::default()
	    },
	)?;
	```

=== "glcli"

	```bash
	glcli signer run --state-signature-mode hard
	```

#### Operator-assisted override

For manual recovery, a signer can be started with an override that
accepts missing and invalid state signatures for the lifetime of the
process. The override has to be acknowledged explicitly
(`I_ACCEPT_OPERATOR_ASSISTED_STATE_OVERRIDE`), and its use is reported
to Greenlight. It disables the authenticity guarantee and should only
be used together with the Greenlight team.

```bash
glcli signer run \
  --state-override I_ACCEPT_OPERATOR_ASSISTED_STATE_OVERRIDE \
  --state-override-note "support case reference"
```

### Freshness and completeness: the state anchor

!!! example "Planned"
	The state anchor is not available yet. Option and function names on
	this page may change before it ships.

Signatures show that an entry was written by a signer of the node, but
not that it is the _latest_ version, or that nothing was left out. A
signer starting from scratch, which on mobile happens at every app
launch, cannot tell an old, validly signed snapshot from a current one.

The _state anchor_ closes that gap. It is a small record, kept on the
device, of the highest version and a short hash of every key the signer
has seen, including tombstones. It is not a copy of the state, and it
contains no keys or secrets.

When a signer process starts and receives its first snapshot, it
compares the snapshot against its anchor:

| Check | Condition | Outcome |
|-------|-----------|---------|
| Rollback | An entry is older than the anchored version | Refused |
| Omission | A key in the anchor is missing from the snapshot | Refused |
| Resurrection | A key the anchor knows as deleted comes back | Refused |
| Discrepancy | An entry has the anchored version but a different value | Accepted, reported |

A validly signed entry at the same or a higher version is always
accepted: only another signer of the node can have produced it. Keys
the anchor has never seen are accepted and are protected by their
signatures.

The anchor only advances to versions the node has sent back to the
signer, never to versions the signer merely wrote. This way a response
that the node failed to store cannot leave the anchor ahead of the
node's state and lock the signer out.

#### Signers without an anchor

A signer without an anchor trusts the first snapshot it receives and
anchors it. This is the case on first start, after [pairing](pairing.md)
a new device, after [recovery](../getting-started/recover.md), after
reinstalling the application, and after the anchor has been cleared.

Losing the anchor, or restoring an older copy of it from a device
backup, is safe: the anchor only ever records versions the node once
had, so an older anchor is merely less protective. The anchor needs
persistent storage, not secret storage.

#### Storing the anchor

`gl-client` itself does not access the filesystem, so that the signer
can also run in environments without one, such as a hardware security
module. It defines a storage interface for the anchor instead, and the
higher-level libraries provide implementations:

| Library | Anchor storage |
|---------|----------------|
| `gl-client` (Rust) | Storage interface only; bring your own implementation |
| `gl-sdk` | File-backed, configured with a path in the application's data directory |
| `gl-client-py`, `glcli` | File-backed, configured with a path |
| `gl-testing` | In memory |

#### Enforcement

Anchor enforcement is a signer option with two values:

 - `report`: the checks run and their results are reported to
   Greenlight, but nothing is refused.
 - `enforce`: rollback, omission and resurrection are refused.

The anchor is only meaningful together with `hard` state signature
mode, because `soft` mode accepts unsigned entries at any version. It
will start out as `report` and move to `enforce` after `hard` mode is
the default.

Like the state signature mode, the enforcement option is chosen by the
application embedding the signer. The Greenlight node cannot change it.

#### Clearing the anchor

If a signer refuses to start because of an anchor check, either the
state was tampered with, or the node's stored state went back in time
for an operational reason, for example a restore from backup. The
signer cannot tell these apart, and there is deliberately no way for
the node to tell it.

Clearing the anchor on the device (`reset_anchor()` in `gl-sdk`,
`--reset-anchor` in `glcli`) makes the signer trust the next snapshot
again, like a signer without an anchor. Applications should only
offer this as an explicit user action, ideally after contacting
Greenlight support.

## Limits

 - A signer without an anchor trusts the first state it receives.
 - The anchor protects what that particular device has seen. A key
   written by another signer, and never seen by this one, is protected
   by its signature only.
 - A device that has been offline for a long time only refuses state
   older than what it saw when it was last online.

[vls]: https://vls.tech
