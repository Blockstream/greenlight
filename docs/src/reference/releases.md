# Releasing the Greenlight SDK

This page documents how a version of the Greenlight SDK is cut and
published, and which automation is (and is not) wired up. The SDK is a
Cargo workspace that is published to several registries at once, so a
release touches multiple pipelines with slightly different triggers and
failure modes.

## What gets published, and by what

| Artifact | Workflow | Trigger | Registry |
| --- | --- | --- | --- |
| `gl-client`, `gl-plugin`, `gl-signerproxy`, `gl-cli`, `gl-sdk`, `gl-sdk-cli` (Rust) | `.github/workflows/release.yml` | push of a `<crate>-vX.Y.Z` tag | crates.io |
| `gl-client` (Python, `libs/gl-client-py`) | `.github/workflows/python.yml` | push of a `gl-client-py-vX.Y.Z` tag | PyPI (trusted publishing) |
| `gl-sdk` (Python, `libs/gl-sdk`) | `.github/workflows/python.yml` | push of a `gl-sdk-vX.Y.Z` tag | PyPI (trusted publishing) |
| `@greenlightcln/glsdk` (npm) | `.github/workflows/typescript.yml` | push to `main` whose `package.json` version is not on npm yet | npm |
| `com.blockstream:glsdk` (Android, JVM) | `gitlab/deploy-maven.yml` on the internal GitLab mirror | manual `publish_to_maven` job on a tag pipeline | Maven Central |
| `GreenlightSDK` (Swift) | `swift.yml` in `Blockstream/gl-sdk-swift` | manual dispatch | GitHub Releases of that repo |
| `clnvm` (Python) | `task clnvm:publish` (manual `twine`) | manual | PyPI |

## Tag conventions

The crates.io workflow fires on any tag matching:

```text
*-v[0-9]+.[0-9]+.[0-9]+
```

so a release tag looks like `gl-client-v0.7.0`. Everything before the
last `-v` is taken as the crate name and the rest as the version. The
workflow then enforces a **"Version Match"** check: the tag version must
equal the `version` in that crate's `Cargo.toml`.

The same pattern also matches the Python release tags. A
`gl-sdk-vX.Y.Z` tag therefore publishes both the `gl-sdk` crate and the
`gl-sdk` Python package. The `gl-client-py` crate sets `publish = false`,
so `release.yml` skips `gl-client-py-vX.Y.Z` tags and only
`python.yml` acts on them. `python.yml` checks that every built wheel
carries the version from the tag, which comes from the package's
`pyproject.toml`.

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
   `gl-plugin`, `gl-signerproxy`, `gl-cli` and `gl-sdk`, then
   `gl-sdk-cli`. If they race, the dependent crate fails
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
| *(none)* | `python.yml` | PyPI, through trusted publishing in the `pypi` environment |
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

- **npm publishes whatever version is missing from the registry.**
  `typescript.yml` builds, tests and publishes when the `package.json`
  version is not in the registry's version list for `@greenlightcln/glsdk`.
  It compared against the parent commit before, which skipped the
  publish after a rebase-merge buried the version bump below the tip,
  and that left npm at 0.1.1 while `package.json` said 0.3.0.
- **PyPI trusted publishing has to be set up on pypi.org.** The publish
  jobs in `python.yml` run in the `pypi` GitHub environment and
  authenticate with the workflow's OIDC token. Each PyPI project needs a
  trusted publisher for owner `Blockstream`, repository `greenlight`,
  workflow `python.yml` and environment `pypi`. For a project that does
  not exist on PyPI yet, add it as a *pending* publisher.
- **The `gl-sdk` Python wheel is retagged.** hatchling builds it as
  `py3-none-any`, but it bundles the native library, so `python.yml`
  retags each wheel with the platform the library was built for. The
  Linux libraries are built with `cargo zigbuild` against glibc 2.17,
  which is what the `manylinux_2_17` tag promises. The bindings in the
  wheel are regenerated from the bundled library, and the job warns if
  the committed `libs/gl-sdk/glsdk/glsdk.py` differs from them.
- **Swift is a separate repository.** `gl-sdk-swift` is published from
  its own repo (`Blockstream/gl-sdk-swift`) and is out of scope here.
