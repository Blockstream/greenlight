# Signer Backups

Greenlight signers can keep a local copy of the VLS signer state to enable disaster recovery or migration to a self-hosted node. This backup is opt-in and disabled by default. When enabled, the backup file contains signer state entries for recoverable channels and known peers.

## Enable backups

Start the signer with a backup path:

```bash
glcli signer run --backup-path backup.json
```

By default, the signer writes a snapshot when a new recoverable channel appears.
For periodic snapshots, use:

```bash
glcli signer run \
  --backup-path backup.json \
  --backup-strategy periodic \
  --backup-periodic-updates 10
```

The backup file is not created immediately at process startup. It is created
after a snapshot trigger, such as a new recoverable channel or the configured
periodic update threshold.

## Backup strategies

`new-channels-only` is the default strategy. It writes a snapshot when a channel
first becomes recoverable, which keeps disk writes low while still capturing the
data needed to recover that channel later.

`periodic` writes the initial snapshot for new recoverable channels and then
writes again after the configured number of recoverable channel updates. Use it
when you want the local file to track ongoing signer-state changes more closely,
at the cost of more frequent disk writes.

If a backup write fails, the signer logs the error and continues. Check signer
logs when relying on local backups.

## Inspect a backup

Use `inspect-backup` to verify that the file can be read and to list the
recoverable channel inventory:

```bash
glcli signer inspect-backup --path backup.json
```

For human-readable output:

```bash
glcli signer inspect-backup --path backup.json --format text
```

Channels with missing `peers/{peer_id}` signer-state entries are marked
incomplete. Incomplete channels remain visible, but they cannot be converted
into complete CLN recovery entries until an address is available.

## Convert for Core Lightning

Convert the signer backup to Core Lightning recovery input:

```bash
glcli signer convert-backup --path backup.json --format cln
```

To write the converted recovery request to a file:

```bash
glcli signer convert-backup \
  --path backup.json \
  --format cln \
  --output cln-recoverchannel.json
```

The output is a CLN `recoverchannel` request body containing `scb` entries:

```json
{
  "scb": ["<hex-encoded-static-channel-backup>"]
}
```

When VLS counterparty revocation secrets are present in the backup, the
converted CLN SCB entries include the shachain TLV. If that signer state is
absent, conversion still emits CLN recovery input without the shachain TLV.

Pass the generated `scb` array to CLN's `recoverchannel` RPC. The exact command
depends on the CLN RPC client you are using. Greenlight does not execute
recovery; `convert-backup` only prepares CLN recovery input.

If the backup contains incomplete channels and you still want to export the
complete ones, use:

```bash
glcli signer convert-backup \
  --path backup.json \
  --format cln \
  --skip-incomplete
```

## Current limitations

- CLN conversion assumes current v1 channels where the channel id is derived
  from the funding outpoint.
- The CLN shachain TLV is included only when VLS counterparty revocation
  secrets are present in the backup.
- Missing `peers/{peer_id}` signer-state entries make affected channels
  incomplete. This issue will gradually resolve as the signer receives peer
  addresses during normal operation.
