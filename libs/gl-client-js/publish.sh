#!/bin/bash -x
set -e
# We need to publish in a temporary directory since we're going to
# rewrite a bunch of paths and directories.
tmpfile=$(mktemp -d /tmp/gl-client-js.XXXXXX)
echo "$tmpfile"
FILES=$(jq '.files[]' -r package.json | grep -vE '(index.node|libhsmd-sys-rs|gl-client)')
cp -R .npmrc $FILES "${tmpfile}"
sed -i 's/gl-client = { path = ".." }/gl-client = { path = "gl-client" }/g' "${tmpfile}/Cargo.toml"
echo '[workspace]' >> "${tmpfile}/Cargo.toml"

# Now vendor the crate we depend on, but stripping its workspace entry:
cp -r ../gl-client "${tmpfile}/"
sed -i '/\[workspace\]/Q' "${tmpfile}/gl-client/Cargo.toml"

cp -r ../libhsmd-sys-rs/ "${tmpfile}/"
sed -i 's|libhsmd-sys = { path = "libhsmd-sys-rs" }|libhsmd-sys = { path = "../libhsmd-sys-rs" }|g' "${tmpfile}/gl-client/Cargo.toml"

(
    cd "${tmpfile}"
    npx google-artifactregistry-auth
    npm publish
)
