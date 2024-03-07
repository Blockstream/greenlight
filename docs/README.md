# Greenlight docs

The docs are hosted on https://blockstream.github.io/greenlight/

## Contributing to the documentation

You must have a working installation of `python` and `poetry` to contribute to the docs.

To install dependencies make sure you are at the root of the repository

```
poetry install --with-only docs
```

To build the docs

```
cd docs; mkdocs build
```

To serve the docs locally
```
cd docs; mkdocs serve
```
