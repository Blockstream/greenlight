## Installing the library
<!-- Installing dependencies --> Greenlight provides client libraries
for a variety of programming languages, providing an idiomatic
interface to developers. The libraries allow interaction with both the
_Scheduler_ and the _Node_. The _Scheduler_ provides access to the
node metadata, while the _Node_ is the user's CLN node running on
Greenlight's infrastructure.

Steps to install the library depend on the programming language and
target environment. The code blocks below provide tabs for the most
common ones:

=== "Rust"
	Add the `gl-client` crate as a dependency:

	```toml
	[dependencies]
	gl-client = { git = "ssh://git@github.com/Blockstream/greenlight" }
	```
	
	
=== "Python"
	The python library currently resides on a private repository which has to be specified during the installation:
	
    ```sh
	pip install \
	  --extra-index-url=https://us-west2-python.pkg.dev/c-lightning/greenlight-pypi/simple/ \
	  -U glcli
	```

=== "Javascript"
	The javascript library currenctly resides on a private repository, which needs to be configured the first time you install it:
	```sh
	npm config set @greenlight:registry=https://us-west2-npm.pkg.dev/c-lightning/test-npm/
	```
	
	Afterwards specifying the namespace `@greenlight` is sufficient to install from the private repository.
	```sh
	npm install @greenlight/gl-client-js
	```

