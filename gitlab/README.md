## Gitlab Pipelines
There are 5 Greenlight packaged binaries/libraries that are published - 1 Python packages and 4 Rust crates. 

The builds are triggered by certain tag formats. Make sure to include a patch as well since the Rust compiler requires that (applicable to `gl-client-py` as well).

Packages and tag formats
---
If a tagged release doesn't work, it can be retried by adding an `_N` suffix after the date part. For example: `..YYYYMMDD_1-x.x.x` or `..YYYYMMDD_25-x.x.x`.

**Note:** The last part of the tag (after the `-`) is used to change the version to be pushed. If different from what's in the respective `pyproject.toml`/`Cargo.toml`, the TOML files will be overwritten via `sed` by the Gitlab job.

### PyPI
* gl-client-py - `glclientpy_YYYYMMDD-x.x.x`

### crates.io
* gl-client - `glclient_YYYYMMDD-x.x.x`
* gl-plugin - `glplugin_YYYYMMDD-x.x.x`
* gl-signerproxy - `glsignerproxy_YYYYMMDD-x.x.x`
* gl-cli -`glcli_YYYYMMDD-x.x.x`
