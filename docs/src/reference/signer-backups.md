# Signer Backups

Greenlight signers can keep a local copy of the VLS signer state to enable disaster recovery or migration to a self-hosted node. This backup is opt-in and disabled by default. When enabled, the backup file contains the signer state plus the peer address list needed to
build channel recovery data.

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

If a backup write or peer-list refresh fails, the signer logs the error and
continues. Check signer logs when relying on local backups.

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

Channels with missing peer addresses are marked incomplete. Incomplete channels
remain visible, but they cannot be converted into complete CLN
recovery entries until an address is available.

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
- Missing peer addresses make affected channels incomplete.
