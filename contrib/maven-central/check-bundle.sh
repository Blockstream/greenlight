#!/usr/bin/env bash
# Checks a com.blockstream:glsdk release bundle and signs it the way Maven
# Central requires.
#
#   check-bundle.sh <bundle.zip> <version> <gpg key id> <signed.zip>
#
# The bundle is what `./gradlew :lib:publishAllPublicationsToBundleRepository`
# writes to libs/gl-sdk-android/lib/build/maven-bundle, zipped: the
# publications in the Maven repository layout, unsigned. This checks the
# layout and the POMs, writes the md5 and sha1 checksums Central requires,
# signs every file with the given key from the gpg keyring, checks the
# signatures and zips the result to <signed.zip>, ready for the Central
# Portal's upload API. MAVEN_SIGNING_KEY_PASSWORD is the key's passphrase,
# if it has one.
#
# The Release workflow runs it with a throwaway key on every release, dry
# run or not, so a bundle that Central would reject never leaves GitHub.
# Runs on macOS (bash 3.2) and Linux.
set -euo pipefail

[ $# = 4 ] || { echo "usage: $0 <bundle.zip> <version> <gpg key id> <signed.zip>" >&2; exit 2; }
ZIP=$(cd "$(dirname "$1")" && pwd)/$(basename "$1")
VERSION=$2 KEY_ID=$3
OUT=$(cd "$(dirname "$4")" && pwd)/$(basename "$4")

digest() { openssl dgst "-$1" "$2" | sed 's/.*= //'; }

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
mkdir "$WORK/in" "$WORK/out"
unzip -q "$ZIP" -d "$WORK/in"
cd "$WORK/in"

# Only the version directories go to Central. Gradle also writes
# maven-metadata.xml one level up, which Central does not take.
DIRS=$(find . -type d -path "./com/blockstream/glsdk*/$VERSION" | sort)
[ -n "$DIRS" ] || { echo "error: no com/blockstream/glsdk*/$VERSION in $ZIP" >&2; exit 1; }
for d in $DIRS; do
  mkdir -p "$WORK/out/$d"
  find "$d" -maxdepth 1 -type f ! -name '*.md5' ! -name '*.sha1' ! -name '*.sha256' ! -name '*.sha512' ! -name '*.asc' \
    -exec cp {} "$WORK/out/$d/" \;
done
cd "$WORK/out"

for d in $DIRS; do
  NAME=$(basename "$(dirname "$d")")
  POM="$d/$NAME-$VERSION.pom"
  [ -f "$POM" ] || { echo "error: $NAME has no $NAME-$VERSION.pom" >&2; exit 1; }
  # What Central requires of every POM.
  for tag in name description url licenses developers scm; do
    grep -q "<$tag>" "$POM" || { echo "error: $POM has no <$tag>" >&2; exit 1; }
  done
  for c in sources javadoc; do
    [ -f "$d/$NAME-$VERSION-$c.jar" ] || echo "warning: $NAME has no $c jar, Central may reject it" >&2
  done
done

FILES=$(find . -type f | sort)
for f in $FILES; do
  digest md5 "$f" > "$f.md5"
  digest sha1 "$f" > "$f.sha1"
  printf '%s' "${MAVEN_SIGNING_KEY_PASSWORD:-}" | gpg --batch --yes --quiet --pinentry-mode loopback --passphrase-fd 0 \
    --local-user "$KEY_ID" --armor --detach-sign --output "$f.asc" "$f"
  gpg --batch --quiet --verify "$f.asc" "$f" 2>/dev/null || { echo "error: the signature of $f does not verify" >&2; exit 1; }
done
echo "Signed $(echo "$FILES" | wc -l | tr -d ' ') files with $KEY_ID:"
echo "$FILES" | sed 's/^/  /'

rm -f "$OUT"
zip -q -r "$OUT" com
echo "com.blockstream:glsdk:$VERSION is ready for Maven Central: $OUT"
