# Register a node

In this section we'll use a developer certificate to register a node.

We'll start with creating a seed that is used to derive node-secrets from. Each
node on the lightning network is identified by a public key and the
corresponding private key is one of these secrets. In the next step, we'll
connect to the _Scheduler_ using a developer identity and register the node.
This requires you to prove that you own the private key mentioned previously.

At the end of this section your node will be registered on Greenlight and you
will have a device-identity that can be used to connect the node.

## Creating a seed

Let's start with the seed secret: the seed secret
is a 32 byte secret that all other secrets and private keys are
derived from, as such it is paramount that this secret never leaves
your user's device and is only handled by the _Signer_.

We suggest to derive the seed secret from a [BIP 39][bip39] seed phrase, so the user
can back it up on a physical piece of paper, steel plate, or whatever
creative way of storing it they can think of.

!!! note
	The following code-snippets build on each other. By copying each snippet
	after the other you should get a working example. See the getting started project in [examples](https://github.com/Blockstream/greenlight/tree/main/examples/rust) to view the code in one file.

=== "Rust"
	
	Install the `bip39` and `rand` crates required for secure randomness and conversion of mnemonics. Add the following lines to your Cargo.toml
	
	```toml
	[dependencies]
	rand = "*"
	bip39 = { version = "*", features=["rand_core"] }
	```

=== "Python"

	Install the `bip39` package which we'll use to encode the
	seed secret as a seed phrase:
	
	```sh
	pip install bip39
	```

Now we can securely generate some randomness, encode it as BIP 39
phrase and then convert it into a seed secret we can use:

=== "Rust"
	```rust
--8<-- "getting_started.rs:create_seed"
	```

=== "Python"

	```python
--8<-- "getting_started.py:create_seed"
	```

!!! important
	Remember to store the seed somewhere (file on disk, registry, etc)
	because without it, you will not have access to the node, and any
	funds on the node will be lost forever! We mean it when we say _you're
	the only one with access to the seed_!

## Initializing the signer

To initialize a signer we'll first need to configure `Nobody` credentials so we can talk to the scheduler using mTLS. Nobody credentials require data from the files downloaded from the Greenlight Developer Console, so the files must be accessible from wherever the node registration program is run. Any connection using the
`developer_creds` object will allow you to register new Greenlight
nodes.

=== "Rust"
	```rust
--8<-- "getting_started.rs:dev_creds"
	```
	
=== "Python"

	```python
--8<-- "getting_started.py:dev_creds"
	```
	

The next step is to create the [`Signer`][signer] which processes incoming signature
requests, and is used when registering a node to prove ownership of
the private key. The last thing to decide is which network we want the
node to run on. You can chose between the following networks:

 - `testnet`
 - `bitcoin`

We'll set NETWORK as `bitcoin`, because ... reckless 😉

=== "Rust"
	```rust
--8<-- "getting_started.rs:init_signer"
	```

=== "Python"
	```python
--8<-- "getting_started.py:init_signer"
	```
	
[bip39]: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki


## Registering a new node

Registering a node with the [`Scheduler`][scheduler] creates the node on the
Greenlight service and ensures everything is setup to start the node.

In order to register a node, the client needs to prove it has access to the
node's private key. Since the private key is managed exclusively by the
[`Signer`][signer] we need to pass the [`Signer`][signer] to the
[`Scheduler`][scheduler]:

=== "Rust"
	```rust
--8<-- "getting_started.rs:register_node"
	```

=== "Python"
	```python
--8<-- "getting_started.py:register_node"
	```

The result of `register` contains the credentials that can be used
going forward to talk to the scheduler and the node itself. 

!!! important 
	Please make sure to store them somewhere safe, since anyone with 
	these credentials can access your node.

=== "Rust"
	```rust
--8<-- "getting_started.rs:get_node"
	```

=== "Python"
	```python
--8<-- "getting_started.py:get_node"
	```

If you get an error about a certificate verification failure when
talking to the node, you most likely are using an unconfigured
`TlsConfig` that doesn't have access to the node. See
[Security][security] for details on how authentication and
authorization work under the hood.


[security]: ../reference/security.md
[signer]: ./index.md#signer
[scheduler]: ./index.md#scheduler
[auth]: ./index.md#authentication
[certs]: ./certs.md
