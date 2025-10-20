# A version manager for Core Lightning binaries

It is sometimes useful to download and install multiple versions of Core Lightning on a single machine.
One use-case is to test your greenlight application. (This is also the only use-case this library supports at the moment).

All versions of Core Lightning are installed in a configurable directory
- `GLTESTING_CLN_PATH` if it is configured
- `$XDG_CACHE_DIR/greenlight` if $XDG_CACHE_DIR is configured
- `~/.cache/greenlight` otherwise

## Usage

To download all binaries 

```
clnvm get-all
```

To download a specific binary

```
clnvm get --tag 23.01
```
