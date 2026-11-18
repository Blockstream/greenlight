## Gitlab Pipelines

The internal GitLab mirror of this repository only builds and publishes
the Android/JVM library, `com.blockstream:glsdk`, to Maven Central.
Every other package is published from GitHub Actions, see
[the release docs](../docs/src/reference/releases.md).

The mirror is updated by pushing `main` and the release tags to it.

### Maven Central

* `build_kotlin` runs on every tag and on `main`, on the `m4` macOS runner.
* `publish_to_maven` is a manual job on tag pipelines. It publishes the
  version in `libs/gl-sdk-android/gradle.properties` (`libraryVersion`),
  not the version in the tag, so run it once per release, on the
  `gl-sdk-vX.Y.Z` tag's pipeline.
* `publish_snapshot_to_maven` publishes a `-SNAPSHOT` build on every
  branch pipeline.

The Maven Central credentials and the GPG signing key are configured in
the runner's `~/.gradle/gradle.properties`.
