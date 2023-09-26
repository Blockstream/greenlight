## Installing the library

Greenlight provides client libraries for a variety of programming
languages, providing an idiomatic interface to developers. The
libraries allow interaction with both the _Scheduler_ and the
_Node_. The _Scheduler_ provides access to the node metadata, while
the _Node_ is the user's CLN node running on Greenlight's
infrastructure.

Steps to install the library depend on the programming language and
target environment. The code blocks below provide tabs for the most
common ones:

=== "Rust"
	Add the `gl-client` crate as a dependency by either editing the
	`Cargo.toml` and add the following lines

	```toml
	[dependencies]
	gl-client = { git = "ssh://git@github.com/Blockstream/greenlight" }
	```

	or by using `cargo add`:

	```bash
	cargo add --git https://github.com/Blockstream/greenlight.git
	```

	Note: the rust library currently relies on `git` dependencies, which
	crates.io does not allow. The `gl-client` library on crates.io is a
	placeholder until our dependencies stabilize.


=== "Python"
	The `gl-client` package is available on the public PyPI:

	```sh
	pip install -U gl-client
	```

	If you use a different dependency management system please see its
	documentation about how to specify `gl-client` as a dependency.

