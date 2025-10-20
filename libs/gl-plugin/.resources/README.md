# Package files

Due to the way `cargo` bundles packages we need to keep a copy of the
shared resources into the directory itself. This directory holds those
copies. The files in here are marked as `derived` in `git` not to
duplicate the `diff` output when reviewing changes to TLS and protobuf
file.

These files are not to be updated manually, rather use the `make`
target in `libs/gl-plugin/Makefile`: `make sync`.
