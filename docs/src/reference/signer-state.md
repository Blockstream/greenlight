# Signer State

The signer is built on the [Validating Lightning Signer][vls] (VLS).
VLS does not sign blindly: before signing it checks the request against
what it knows about the node, for example the current state of each
channel, which destinations the on-chain wallet may send to, and which
invoices and preimages it has seen. This knowledge is the _signer
state_.

Greenlight signers do not keep a durable copy of the full signer state,
which is what allows a signer to run on a phone, in a short-lived process,
or on several devices at once. Instead, the Greenlight node stores the
signer state and attaches it to every request it sends to the signer.
With persistent device storage, the signer keeps a watermark
independently of the node.

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
| `key` | The entry name, prefixed by its class: `nodes/`, `nodestates/`, `channels/`, `allowlists/`, `trackers/` |
| `version` | A counter that increases with every change to this key |
| `value` | The serialized VLS object |
| `signature` | The signer's signature over the key, version and value |

The node also sends `peers/` entries as address hints for channel backup
recovery. These are untrusted metadata. The signer keeps valid hints for
backups and excludes them from state signatures and the watermark.

Deleting a key leaves a _tombstone_ behind: the entry is kept with the
highest possible version and an empty value, so that an older value of
the key cannot be merged back in.

## Synchronization

The node and the signer keep their copies of the state in sync by
exchanging only what changed:

 1. When a signer connects, the node sends it a heartbeat request that
    carries a full snapshot of the state, including tombstones. The signer
    checks the first snapshot after startup against the saved watermark.
 2. Every following request carries only the entries that changed
    since the node last sent state to that signer.
 3. The signer merges incoming entries into a scratch copy of its state.
    New keys are added, higher versions replace lower ones, lower
    versions are ignored.
 4. The signer authorizes and processes the request against that copy.
    Only after processing and state signing succeed does it commit the
    copy and return the signed changes. It saves the watermark before
    committing the copy or returning signatures. A request rejected
    before the save leaves both live state and the watermark unchanged.
 5. The node merges and persists the returned entries _before_ it
    hands the signer's response on. If the entries cannot be stored,
    the response is dropped, so the node never acts on a signature whose
    state was lost.

Builds with the `permissive` feature report authorization failures but
continue processing them. Requests that return an error still leave the
live state unchanged.

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
| Freshness | Is any entry older than one this signer has already seen? | Device watermark | With persistent device storage |
| Completeness | Is any entry this signer has seen missing? | Device watermark | With persistent device storage |

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
| `soft` | Accepted for one successfully processed snapshot, and signed on the way out | Refused | One-time migration of nodes whose state predates state signatures |
| `hard` | Refused | Refused | Production (default) |

`hard` is the default. A signer never automatically falls back to
`soft` when it encounters unsigned state. Only `hard` mode guarantees
that every entry the signer acts on was written by a signer of this node,
provided the operator override is disabled.

The mode is part of the signer configuration:

=== "Rust"

	```rust
	use gl_client::signer::{FileDeviceStorage, Signer, SignerConfig};
	use std::sync::Arc;

	let storage = Arc::new(FileDeviceStorage::new("/path/to/app/device-state")?);
	let signer = Signer::new_with_config(seed, network, creds, SignerConfig {
	    device_storage: Some(storage),
	    ..Default::default()
	})?;
	```

=== "glcli"

	```bash
	glcli signer run --state-signature-mode hard
	```

#### One-time migration

An application can explicitly select `soft` to migrate a legacy node.
One successfully processed state snapshot can run in Soft mode. Missing
signatures in that snapshot are accepted and repaired; invalid signatures
are refused. After a successful migration, all subsequent requests use Hard, including after
reconnecting. Clones of the signer share this transition.

The signer saves a migration marker in device storage after processing
and signing succeed, then saves the watermark before returning signatures.
The marker and watermark are scoped by node ID and network. If either is
already present, requesting Soft starts the signer in Hard with a warning.
Applications can keep the same configuration across restarts. A
successfully processed empty snapshot also consumes the allowance.

`DeviceStorage` provides fallible `load` and `save` operations over opaque
bytes; writes must replace values atomically and durably. Each purpose has
its own key and encoding.
Storage errors prevent the migration from publishing state or signatures.
If writing the marker fails, the current signer switches to Hard because
the failed write may still have persisted the marker.

=== "glcli"

	```bash
	glcli signer run --state-signature-mode soft
	```

=== "Python"

	```python
	signer = glclient.Signer(
	    seed, network, creds,
	    state_signature_mode="soft",
	    device_storage_path="/path/to/app/device-state",
	)
	```

=== "gl-sdk (Python)"

	```python
	migration = glsdk.SignerConfig(
	    state_signature_mode=glsdk.StateSignatureMode.SOFT,
	    device_storage_path="/path/to/app/device-state",
	)
	config = glsdk.Config().with_signer_config(migration)
	node = glsdk.NodeBuilder(config).connect(credentials, mnemonic)
	```

	Standalone signers use `Signer.new_with_config(mnemonic, migration)`
	or `Signer.new_from_seed_with_config(seed, migration)`. Authenticating
	a signer preserves its migration configuration and storage.

The migration deliberately trusts the unsigned snapshot it migrates. If the
process crashes or the connection drops after saving the marker but
before the node persists the repaired state, the allowance stays
consumed. Restarting with Soft uses Hard; it does not retry the migration.
Contact Greenlight support for explicit recovery if the node still has
unsigned state.

#### Operator-assisted override

For manual recovery, a signer can be started with an override that
accepts missing and invalid state signatures for the lifetime of the
process. The override has to be acknowledged explicitly
(`I_ACCEPT_OPERATOR_ASSISTED_STATE_OVERRIDE`), and its use is reported
to Greenlight. It disables the authenticity guarantee while watermark
checks continue to apply. Use it only together with the Greenlight team.

```bash
glcli signer run \
  --state-override I_ACCEPT_OPERATOR_ASSISTED_STATE_OVERRIDE \
  --state-override-note "support case reference"
```

### Freshness and completeness with the watermark

The watermark records the version and hash of each value in the last
committed state. It includes tombstones and covers only state committed
by this signer. At startup, the signer rejects a snapshot that omits an
anchored key, lowers its version, changes its value at the same version,
or brings a deleted key back. Once initialized, it can merge incremental
updates with newer state already in memory.

The signer saves the watermark before releasing any response that
depends on a state change. If the storage write fails, the signer stops
processing requests until it is reconstructed and reloads storage. A
watermark violation also stops the signer loop and surfaces an error.

A response lost before the node saves it can leave the watermark ahead
of the node. Two devices can also produce different values at the same
version. After restart, the saved watermark requires the node to supply
a snapshot that satisfies it. The signer has no automatic recovery for
missing or conflicting state.

#### Manual recovery in Hard mode

First try to restore a node snapshot that satisfies the saved watermark.
If that snapshot is unavailable, an operator of a Hard-only app can
explicitly accept a new baseline after independently verifying the state
the node will supply. Stop the signer, back up the device storage, and
remove only the `signer/{network}/{node_id}/watermark` record. For
`FileDeviceStorage`, the filename is the hex SHA-256 hash of that key,
using the hex node ID. Keep the Soft migration marker intact.

Restart in Hard mode. The first accepted signed snapshot establishes the
new baseline. Resetting the watermark discards the ability to reject a
previously signed snapshot based on the old baseline.

#### Configuring device storage

Soft mode requires persistent storage. In Hard mode, set
`SignerConfig::device_storage` in Rust to enable the watermark. Reuse the
same storage across restarts. Unreadable storage and invalid watermark
data prevent signer initialization.

Upgrade the node plugin first so it includes tombstones in initial
snapshots. Off mode is intended for development and bypasses both
signature verification and the watermark.

#### Limits

A device without a watermark uses the first accepted snapshot as its
baseline, including after pairing or reinstalling. Deleting or rolling back
device storage can weaken both rollback protection and the Soft migration
limit.

Run one active signer instance per device storage namespace. Clones
share the watermark, but separate signer instances and devices cannot
coordinate their local records. We plan to address coordination with an
external watermarking service outside Blockstream’s control.

[vls]: https://vls.tech
