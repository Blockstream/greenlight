#!/usr/bin/env bash
# Checks the files about to go to PyPI: each one carries the expected
# version, and twine accepts the metadata. Used by python.yml in dry runs
# and real releases alike.
#
#   check-dist.sh <distribution name, e.g. gl_client> <version> <dir>
set -euo pipefail
NAME=$1 VERSION=$2 DIR=$3
ls -la "$DIR"
[ -n "$(ls -A "$DIR")" ] || { echo "::error::No files in $DIR"; exit 1; }
for f in "$DIR"/*; do
  case "$(basename "$f")" in
    "$NAME-$VERSION"-*.whl | "$NAME-$VERSION".tar.gz) ;;
    *) echo "::error::$(basename "$f") is not $NAME $VERSION"; exit 1 ;;
  esac
done
pipx run twine check --strict "$DIR"/*
