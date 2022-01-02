#!/bin/bash

# We need to publish in a temporary directory since we're going to
# rewrite a bunch of paths and directories.
tmpfile=$(mktemp -d /tmp/gl-client-js.XXXXXX)
echo "$tmpfile"
FILES=$(jq '.files[]' -r package.json | grep -v index.node)
cp -R .npmrc $FILES "${tmpfile}"
sed -i 's/gl-client = { path = ".." }/gl-client = { path = "gl-client" }/g' "${tmpfile}/Cargo.toml"
echo '[workspace]' >> "${tmpfile}/Cargo.toml"

# Now vendor the crate we depend on, but stripping its workspace entry:
mkdir "${tmpfile}/gl-client"
cp ../Cargo.toml "${tmpfile}/gl-client/"
cp ../build.rs "${tmpfile}/gl-client/"
cp -R ../src "${tmpfile}/gl-client/"
cp -R ../tls "${tmpfile}/gl-client/"
cp -R ../proto "${tmpfile}/gl-client/"
sed -i '/\[workspace\]/Q' "${tmpfile}/gl-client/Cargo.toml"

mkdir "${tmpfile}/libhsmd-sys-rs"

cp ../libhsmd-sys-rs/Cargo.toml "${tmpfile}/libhsmd-sys-rs/"
cp ../libhsmd-sys-rs/build.rs "${tmpfile}/libhsmd-sys-rs/"
cp ../libhsmd-sys-rs/libhsmd.[ch] "${tmpfile}/libhsmd-sys-rs/"
cp ../libhsmd-sys-rs/shims.c "${tmpfile}/libhsmd-sys-rs/"
cp -R ../libhsmd-sys-rs/src "${tmpfile}/libhsmd-sys-rs/"
sed -i 's|libhsmd-sys = { path = "libhsmd-sys-rs" }|libhsmd-sys = { path = "../libhsmd-sys-rs" }|g' "${tmpfile}/gl-client/Cargo.toml"

(
    cd "${tmpfile}"
    npx google-artifactregistry-auth
    npm publish
)
