# Greenlight docs

The docs are hosted on https://blockstream.github.io/greenlight/

## Contributing to the documentation

You must have a working installation of `python` and `uv` to contribute to the docs.

To install dependencies make sure you are at the root of the repository

```
uv sync --package gl-docs
```

To build the docs

```
cd docs; uv run mkdocs build
```

To serve the docs locally
```
cd docs; uv run mkdocs serve --verbose --livereload
```
