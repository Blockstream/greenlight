# Client Libraries

We provide a number of client libraries and bindings for a variety of
languages. The core logic is implemented in a small Rust crate called
`gl-client`, on top of which the language bindings are built. Since
these make use of native code they have to build a binary extension
that can be loaded by the language runtime.

We precompile the binaries for a couple of platforms and architectures
to simplify installation for developers. These are automatically
downloaded as part of the installation of the library and should your
combination be supported you will _not_ need any compiler or
sophisticated build process, it should just work out of the box.

## Platforms and Architectures

Currently our CI builds prebuilt bindings for Python for the following
platform and architecture combinations:

| OS      | Architecture   |
|---------|----------------|
| MacOS   | x86_64 (intel) |
| MacOS   | arm64 (m1/m2)  |
| Windows | x86            |
| Window  | x64            |
| Linux   | x86_64-gnu     |
| Linux   | i686-gnu       |
| Linux   | armv7-gnueabi  |
| Linux   | aarch64        |

Should your platform and architecture not be in the list above, don't
worry, you can still build them from the source by checking out the
[repository][repo], and then either call `make build-py` to build just
the bindings you need.

Please let us know if we're missing a combination, so we can try to
add it to our build system, and remove the need to manually compile
the extension going forward.

There is an automated way of doing this in both languages, based on
the source tarball published to NPM and PyPI, however since the build
depends on a sibling crate which is not bundled it will most likely
fail to build from source tarball (we are working on fixing this).

[repo]: https://github.com/Blockstream/greenlight
