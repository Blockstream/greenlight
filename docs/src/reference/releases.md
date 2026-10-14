# Releasing the Greenlight SDK

This page documents how a version of the Greenlight SDK is cut and
published, and which automation is (and is not) wired up. The SDK is a
Cargo workspace that is published to several registries at once, so a
release touches multiple pipelines with slightly different triggers and
failure modes.

## What gets published, and by what

| Artifact | Workflow | Trigger | Registry |
| --- | --- | --- | --- |
| `gl-client`, `gl-sdk`, `gl-sdk-cli` (Rust) | `.github/workflows/release.yml` | push of a `*-v*` tag | crates.io |
| `@greenlightcln/glsdk` (npm) | `.github/workflows/typescript.yml` | push to `main`, gated on a version change | npm |
| `gl-client-py`, `gl-sdk` (Python wheels) | `.github/workflows/python.yml` | push to `main` / PR | **built only — not published** |
| `clnvm` (Python) | `task clnvm:publish` (manual `twine`) | manual | PyPI |

!!! warning
    The Python SDK wheels are built and tested in CI but there is
    currently **no publish step** for them. Publishing them, if it is
    ever wanted, still has to be wired up (see
    [Gaps and gotchas](#gaps-and-gotchas)).

## Tag conventions

The crates.io workflow fires on any tag matching:

```text
*-v[0-9]+.[0-9]+.[0-9]+
```

so a release tag looks like `gl-client-v0.7.0`. Everything before the
last `-v` is taken as the crate name and the rest as the version. The
workflow then enforces a **"Version Match"** check: the tag version must
equal the `version` in that crate's `Cargo.toml`.

## Cutting a release

1. **Bump versions and changelogs.** For each crate going out, set
   `version` in its `Cargo.toml` (and `package.json` for the npm
   binding), and move the `Unreleased` section of its `CHANGELOG.md`
   under a dated version heading.
2. **Merge to `main`.**
3. **Publish the Rust crates** by pushing one semver tag per crate, on
   the commit whose `Cargo.toml` matches:

   ```bash
   git tag -a gl-client-v0.7.0 <commit>
   git tag -a gl-sdk-v0.5.0    <commit>
   git tag -a gl-sdk-cli-v0.4.0 <commit>
   git push origin --tags
   ```

4. **Publish in dependency order.** The higher-level crates depend on
   the lower-level ones, so publish (or re-run) `gl-client` first, then
   `gl-sdk`, then `gl-sdk-cli`. If they race, the dependent crate fails
   with `failed to select a version for the requirement gl-client = ...`
   simply because its dependency is not on crates.io yet. The publish
   step is idempotent — anything already on crates.io is skipped — so a
   re-run after the dependency lands is enough; the tags do not need to
   be re-pushed.

5. **The npm binding** is published automatically by `typescript.yml` on
   the merge push (see the gotcha below).

## Cargo manifest requirements

`cargo publish` is strict about metadata. A missing `license` (or
`repository`) field makes the publish fail with:

```text
missing or empty metadata fields: license
```

Every crate that is meant to go to crates.io must carry:

```toml
[package]
license = "MIT"
repository = "https://github.com/Blockstream/greenlight"
```

(As of the `v26.06gl1` release, `gl-sdk-cli` was missing these and
needed a small follow-up before it would publish.)

## Secrets

Publishing reads these GitHub Actions secrets (in
`Blockstream/greenlight`):

| Secret | Consumed by | Notes |
| --- | --- | --- |
| `CRATES_IO_TOKEN` | `release.yml` | crates.io publishing |
| `NPM_TOKEN` | `typescript.yml` | npm publishing |
| `TWINE_PASSWORD` | manual `task clnvm:publish` only | PyPI; not used by CI |
| `CARGO_REGISTRY_TOKEN` | *none* | orphaned — `release.yml` maps the env var from `CRATES_IO_TOKEN` instead |
| `RELEASE_PLZ_TOKEN` | *none* | orphaned — `release-plz.toml` exists but no workflow runs release-plz |

Long-lived tokens must be **rotated roughly every 90 days**, or
immediately on any auth failure such as a `403 authentication failed`
from `cargo publish`. To rotate a crates.io token: sign in to crates.io
as an owner of *all* published crates, create a token with
`publish-new` + `publish-update` (leave the IP prefix blank, since the
runners use dynamic IPs), then update `CRATES_IO_TOKEN` and re-publish
in dependency order.

## Gaps and gotchas

- **npm publish skips after a rebase-merge.** `typescript.yml` decides
  whether to publish by diffing `package.json` against its immediate
  parent (`HEAD^`). A normal PR merge puts the version bump on the tip,
  so it fires; a **rebase-merge buries the bump below the tip**, the diff
  sees "unchanged", and `check-version` skips build, test **and** publish.
  A `workflow_dispatch` does not help, because `publish` transitively
  depends on the skipped `test` job. The robust fix is to compare against
  the version already published to npm (`npm view @greenlightcln/glsdk
  version`) rather than `HEAD^`.
- **Python wheels are not published.** `python.yml` builds them and
  `import glclient` smoke-tests them, but nothing uploads them to PyPI.
  If Python releases are wanted, add a publish job (ideally PyPI trusted
  publishing, which needs no token) triggered by `gl-client-py-v*` /
  `gl-sdk-v*` tags.
- **Swift is a separate repository.** `gl-sdk-swift` is published from
  its own repo (`Blockstream/gl-sdk-swift`) and is out of scope here.
