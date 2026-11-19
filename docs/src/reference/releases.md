# Releasing the Greenlight SDK

This page documents how a version of the Greenlight SDK is cut and
published. The SDK is a Cargo workspace with bindings for several
languages, published to five registries. One workflow releases all of
them: **Actions > Release > Run workflow** in `Blockstream/greenlight`.

## What gets published, and by what

| Group | Artifact | Version from | Registry | Workflow |
| --- | --- | --- | --- | --- |
| crates | `gl-client`, `gl-util`, `gl-plugin`, `gl-signerproxy`, `gl-cli`, `gl-sdk`, `gl-sdk-cli` | each `Cargo.toml` | crates.io | `publish-crates.yml` |
| python | `gl-client` (`libs/gl-client-py`) | its `pyproject.toml` | PyPI | `python.yml` |
| python | `gl-sdk` (`libs/gl-sdk`) | its `pyproject.toml` | PyPI | `python.yml` |
| npm | `@greenlightcln/glsdk` | `libs/gl-sdk-napi/package.json` | npm | `typescript.yml` |
| swift | `GreenlightSDK` | the `gl-sdk` crate | tag `X.Y.Z` in `Blockstream/gl-sdk-swift` | `swift.yml` |
| maven | `com.blockstream:glsdk` | `libraryVersion` in `libs/gl-sdk-android/gradle.properties` | Maven Central | `kotlin.yml` |
| | `clnvm` (Python) | | PyPI | manual `task clnvm:publish` |

`release.yml` calls the other workflows. On their own, on pushes and
pull requests, they only build and test. Pushing a release tag no
longer publishes anything.

## Cutting a release

1. **Bump versions and changelogs.** Set the new version of everything
   going out, and move the `Unreleased` section of each `CHANGELOG.md`
   under a dated version heading. The plan job enforces that the
   bindings of one SDK release agree:
   - `libs/gl-sdk/pyproject.toml` and `libraryVersion` match the
     `gl-sdk` crate; the Swift package always takes the crate's version;
   - `libs/gl-client-py/pyproject.toml` matches the `gl-client-py`
     crate, and `package.json` matches the `gl-sdk-node` crate.
2. **Merge to `main`.**
3. **Run a dry run**: Actions > Release > Run workflow, with *Dry run*
   ticked (the default). It runs from any branch, so a release can also
   be tried before merging.
4. **Run the release** from `main` with *Dry run* unticked. Untick the
   groups that should not go out.

The plan job asks each registry whether the version from the
repository is published and writes a table of what the run will do to
the run's summary page. A real release publishes only what is missing,
so re-running a partly failed release finishes it. A dry run exercises
every selected artifact, published or not, so it can run at any time,
for example to check that the release pipeline still works.

The crates go first, the bindings after them in parallel, so a binding
never ships a `gl-sdk` version whose crates failed to publish.

## What a dry run checks

Every publication step has a dry-run mode that stops right before the
upload:

| Group | Dry run | Real release adds |
| --- | --- | --- |
| crates | changelog format and entry, `cargo semver-checks` against the last release, tests, and `cargo publish --dry-run` of all crates together. Cargo builds each against the others' packages through a local registry overlay, so this passes before any of them is on crates.io. | the upload in dependency order, and a `<crate>-vX.Y.Z` GitHub release per crate |
| python | the wheels and sdist are built and smoke tested, and carry the expected version; `twine check --strict` | the upload, and a `gl-client-py-vX.Y.Z` GitHub release |
| npm | the native modules are built for every platform and tested; `npm publish --dry-run` | the upload |
| swift | the XCFramework is built; gl-sdk-swift's `scripts/prepare.sh` renders, commits and tags the package locally, and `scripts/check-package.sh` compiles it for the iOS simulator against the fresh zip | the assets on the `gl-sdk-vX.Y.Z` release, then a wait for gl-sdk-swift to tag `X.Y.Z` and a check of it the way SwiftPM installs it |
| maven | the library is built and tested, and the Maven Central bundle is built and run through `contrib/maven-central/check-bundle.sh` with a throwaway key: POM fields, checksums, every file signed and verified | the unsigned bundle on the `gl-sdk-vX.Y.Z` release, then a wait until it is on repo1.maven.org |

The plan also catches what only a registry would reject otherwise, such
as a wildcard (`"*"`) dependency requirement, which crates.io refuses
on upload. To see the plan without running anything:

```bash
python3 .github/scripts/release_plan.py --dry-run
```

## The Swift package

SwiftPM resolves packages by repository, so the package lives in
`Blockstream/gl-sdk-swift` while greenlight builds it. Neither
repository writes to the other, so no cross-repository token is needed:

1. `swift.yml` attaches `glsdkFFI.xcframework.zip`, its checksum and
   `glsdk.swift` to the `gl-sdk-vX.Y.Z` release here.
2. `publish.yml` in gl-sdk-swift polls hourly for such releases, commits
   a `Package.swift` whose binary target points at the zip here, and
   tags and releases `X.Y.Z` there.
3. `swift.yml` waits for that tag, for up to two hours, and verifies it.
   To skip the wait, run `publish.yml` in gl-sdk-swift by hand.

`swift.yml` runs gl-sdk-swift's scripts from its `main` branch in every
build, so changes to the templates there are tested here. GitHub
disables a scheduled workflow after 60 days without activity in its
repository; if a release waits forever, re-enable `publish.yml` on
gl-sdk-swift's Actions tab.

## The Maven package

GitHub holds no Maven Central credentials or signing key. `kotlin.yml`
builds the publications, unsigned, in the Maven repository layout
(`./gradlew :lib:publishAllPublicationsToBundleRepository`), checks them
with `contrib/maven-central/check-bundle.sh` and a throwaway key, and
attaches them to the `gl-sdk-vX.Y.Z` release as `glsdk-maven-X.Y.Z.zip`
with its `.sha256`. Blockstream's release infrastructure picks the
bundle up within the hour, signs it with Blockstream's key and uploads
it. `kotlin.yml` waits, for up to three hours, until the version is on
repo1.maven.org.

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

Every workspace crate is released unless it sets `publish = false`, as
`gl-examples`, `uniffi-bindgen`, `gl-client-py` and `gl-sdk-node` do.

## Secrets and registry setup

| Secret or setting | Used by | Notes |
| --- | --- | --- |
| `CRATES_IO_TOKEN` | `publish-crates.yml` | crates.io publishing |
| `NPM_TOKEN` | `typescript.yml` | npm publishing |
| PyPI trusted publisher | `python.yml` | per PyPI project: owner `Blockstream`, repository `greenlight`, workflow **`release.yml`**, environment `pypi`. The OIDC token names the workflow that started the run, not the called `python.yml`. For a project not on PyPI yet, add a *pending* publisher. |
| `TWINE_PASSWORD` | manual `task clnvm:publish` only | PyPI; not used by CI |
| `CARGO_REGISTRY_TOKEN` | *none* | orphaned, `publish-crates.yml` maps the env var from `CRATES_IO_TOKEN` instead |
| `RELEASE_PLZ_TOKEN` | *none* | orphaned, `release-plz.toml` exists but no workflow runs release-plz |

Long-lived tokens must be **rotated roughly every 90 days**, or
immediately on any auth failure such as a `403 authentication failed`
from `cargo publish`. To rotate a crates.io token: sign in to crates.io
as an owner of *all* published crates, create a token with
`publish-new` + `publish-update` (leave the IP prefix blank, since the
runners use dynamic IPs), then update `CRATES_IO_TOKEN` and re-run the
release.

## Gaps and gotchas

- **The `gl-sdk` Python wheel is retagged.** hatchling builds it as
  `py3-none-any`, but it bundles the native library, so `python.yml`
  retags each wheel with the platform the library was built for. The
  Linux libraries are built with `cargo zigbuild` against glibc 2.17,
  which is what the `manylinux_2_17` tag promises. The bindings in the
  wheel are regenerated from the bundled library, and the job warns if
  the committed `libs/gl-sdk/glsdk/glsdk.py` differs from them.
