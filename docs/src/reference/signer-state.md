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
On the device, the signer keeps a small record called the
[watermark](#freshness-and-completeness-the-watermark). It records the
version and value hash of each signer-owned entry in the last committed
state so the signer can detect stale or incomplete state after restart.

Storing the state on the node creates a tension with the
[security model](security.md). The signer protects funds against an
attacker with access to the infrastructure that stores the state it
relies on. This page describes how the signer can use state it
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
    carries a full snapshot of the state, including tombstones. A newly
    started signer checks this first snapshot against its watermark.
 2. Every following request carries only the entries that changed
    since the node last sent state to that signer.
 3. The signer merges incoming entries into a scratch copy of its state.
    New keys are added, higher versions replace lower ones, lower
    versions are ignored.
 4. The signer authorizes and processes the request against that copy.
    Only after processing and state signing succeed does it save the
    watermark, commit the copy and return the signed changes, in that
    order. If anything fails before the watermark is saved, both the live
    state and the watermark stay unchanged.
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

Run one active signing device per node when using a watermark. If a phone
and desktop sign concurrently, both receive each request. The node uses
the first response and merges state from both signers by key.

Two signers can produce different values at the same version. The node
keeps the first value it stored, and a running signer keeps the value it
already holds at that version. Different signatures over the same value
do not cause a conflict. After restart, the device whose value the node
did not keep can refuse the snapshot. See
[When legitimate state is refused](#when-legitimate-state-is-refused).

## Protecting the state

There are three properties the signer needs from the state it receives:

| Property | Question | Mechanism | Status |
|----------|----------|-----------|--------|
| Authenticity | Was every entry written by a signer of this node? | State signatures | Available |
| Freshness | Is any entry older than one this signer has already seen? | Device watermark | Hard and Soft modes |
| Completeness | Is any entry this signer has seen missing? | Device watermark | Hard and Soft modes |

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
	use gl_client::signer::{FileDeviceStorage, Signer, SignerConfig, StateSignatureMode};
	use std::sync::Arc;

	let storage = Arc::new(FileDeviceStorage::new("/path/to/app/device-state")?);
	let signer = Signer::new_with_config(seed, network, creds, SignerConfig {
	    state_signature_mode: StateSignatureMode::Hard,
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
are refused. After a successful migration, all subsequent requests use
Hard, including after reconnecting. Clones of the signer share this
transition.

The signer saves a migration marker in device storage after processing
and signing succeed, before it returns state or signatures. If the marker
or a watermark already exists for the node, requesting Soft starts the
signer in Hard with a warning. A watermark means that this device has
already accepted state, so there is nothing left to migrate. Applications
can therefore keep the same configuration across restarts. A successfully
processed empty snapshot also consumes the allowance.

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
to Greenlight. It disables the authenticity guarantee, but the watermark
checks still apply. Use it only together with the Greenlight team.

```bash
glcli signer run \
  --state-override I_ACCEPT_OPERATOR_ASSISTED_STATE_OVERRIDE \
  --state-override-note "support case reference"
```

### Freshness and completeness: the watermark

A valid signature proves that a signer of this node wrote an entry. It
does not prove that the entry is the latest version, or that the node
sent every entry. A signer that starts from scratch, which on a phone
happens at every app launch, cannot tell an old, validly signed snapshot
from the current one.

The _watermark_ closes this gap. Like a high-water mark, it records the
highest point the state has reached on this device, and the signer
refuses state that falls below it. It is a small record in device
storage with one line per state entry: the key, its latest version, and
a hash of its value. It includes tombstones. It contains no secrets and
no copy of the state itself.

The signer saves a new watermark with every state change it commits
(step 4 of [Synchronization](#synchronization)). When a newly started
signer receives its first snapshot, it compares the snapshot with the
saved watermark and refuses it on any of these:

| Check | The snapshot... |
|-------|-----------------|
| Rollback | has an entry at a lower version than the watermark, including a deleted key that comes back |
| Omission | is missing a key that the watermark records |
| Conflict | has an entry at the watermark's version, but with a different value |

Entries at higher versions, and keys the watermark has never seen, are
accepted and protected by their signatures. After the first snapshot,
requests carry only changes, and the signer merges them into its
in-memory state, which already satisfies the watermark.

A refused snapshot stops the signer with an error. So does a failed
watermark write, because the write may have partly succeeded. After a
failed write, create a new signer instance so that it reloads the
watermark from storage.

#### When legitimate state is refused

The watermark can also refuse state that nobody tampered with:

 - The signer saves the watermark and sends its response, but the
   response is lost before the node stores the new state. If the signer
   process then restarts, its watermark is ahead of the node.
 - Two devices change the same entry starting from the same version.
   The device whose change the node did not keep now has a watermark
   that conflicts with the node.

The signer cannot distinguish these cases from an attack. Restart can
succeed once the node supplies state that satisfies the watermark. For
a conflict, a higher version of each conflicting entry also satisfies
the check.

#### Resetting the watermark

The preferred fix is to restore node state that satisfies the watermark.
If that is not possible, and you have confirmed that the node's current
state is legitimate, reset the watermark so that the signer accepts that
state as a new baseline:

 1. Stop the signer and back up its device storage.
 2. Remove only the record with the key
    `signer/{network}/{node_id}/watermark`, for example
    `signer/bitcoin/02ab…/watermark`, with the node ID in hex.
    `FileDeviceStorage` names each file after the hex SHA-256 hash of
    its key:

    ```bash
    printf '%s' 'signer/bitcoin/<node_id>/watermark' | shasum -a 256
    ```

    Leave the Soft migration marker in place.

 3. Restart the signer in Hard mode, so that the reset cannot reopen the
    Soft migration. The first signed snapshot it accepts becomes the new
    baseline, even if it is older than the state the old watermark
    recorded.

#### Configuring device storage

Hard and Soft require persistent device storage, which holds the
watermark and the Soft migration marker for each node and network. Reuse
the same storage across restarts. The signer does not start if storage is
not configured or cannot be read, or if the watermark is invalid. Off mode
skips both signature verification and the watermark, and needs no
storage.

| Entry point | Where to supply storage |
|-------------|-------------------------|
| Rust | Pass an `Arc<dyn DeviceStorage>` to `Signer::new` or set `SignerConfig::device_storage`. The optional `file-storage` feature provides `FileDeviceStorage`. |
| gl-client Python | Set the `device_storage_path` keyword on `Signer`. |
| gl-sdk | Pass the path to the standalone constructors, or set `SignerConfig.device_storage_path` through `Config.with_signer_config` before using `NodeBuilder` with a signer. |
| Node.js SDK | Pass the path as the second argument to `Signer`. |
| glcli and gl-sdk-cli | Both use `<data-dir>/device-state`. |

A custom `DeviceStorage` provides fallible `load` and `save` operations
over opaque bytes. Writes must replace values atomically and durably.

The node plugin must include tombstones in the first snapshot. Older
plugins left them out, which the signer refuses as an omission once its
watermark records a deleted key.

#### Limits

 - A device without a watermark trusts the first snapshot it accepts and
   makes it the baseline. This happens on first start, after pairing,
   reinstalling or a reset, and whenever device storage is deleted.
   Restoring an older copy of device storage weakens protection in the
   same way. Either can also reopen the Soft migration.
 - The watermark only covers what this device has seen. An entry written
   by another signer that never reached this device is protected by its
   signature alone.
 - Run one signer instance per device storage. Clones of a signer share
   its watermark, but separate instances and separate devices keep
   independent watermarks. We plan to coordinate them through an external
   watermarking service outside Blockstream's control.

[vls]: https://vls.tech
