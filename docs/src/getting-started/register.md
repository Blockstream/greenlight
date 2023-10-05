# Register a node
## Preparing a node identity

We start by creating a node identity, consisting of a node's seed
secret, and it's mTLS certificates we'll later use to authenticate
against Greenlight. Let's start with the seed secret: the seed secret
is a 32 byte secret that all other secrets and private keys are
derived from, as such it is paramount that this secret never leaves
your device and is only handled by the _Signer_. It is suggested to
derive the seed secret from a [BIP 39][bip39] seed phrase, so the user
can back it up on a physical piece of paper, steel plate, or whatever
creative way of storing it they can think of.

!!! note
	The following code-snippets build on each other. By copying each snippet
	after the other you should get a working example.

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
	use bip39::{Mnemonic, Language};

	let mut rng = rand::thread_rng();
	let m = Mnemonic::generate_in_with(&mut rng, Language::English, 24).unwrap();
	let phrase = m.word_iter().fold("".to_string(), |c, n| c + " " + n);
	
	// Prompt user to safely store the phrase
	
	let seed = m.to_seed("")[0..32];  // Only need the first 32 bytes

	let secret = seed[0..32].to_vec();

	// Store the seed on the filesystem, or secure configuration system
	```

=== "Python"

	```python
	import bip39
	import secrets  # Make sure to use cryptographically sound randomness
	
	rand = secrets.randbits(256).to_bytes(32, 'big')  # 32 bytes of randomness
	phrase = bip39.encode_bytes(rand)
	
	# Prompt user to safely store the phrase
	
	seed = bip39.phrase_to_seed(phrase)[:32]  # Only need 32 bytes
	
	# Store the seed on the filesystem, or secure configuration system
	```

!!! important 
	Remember to store the seed somewhere (file on disk, registry, etc)
	because without it, you will not have access to the node, and any
	funds on the node will be lost forever! We mean it when we say _you're
	the only one with access to the seed_!


Next we instantiate an mTLS identity we will use to authenticate with
Greenlight. Since we haven't registered our node with the service yet,
we will use a [dummy key][auth], that allows us to talk to the 
[`Scheduler`][scheduler] but will not allow us to talk to any other service.
No worries, once we register the node we will get a valid certificate 
to authenticate.

=== "Rust"
	```rust
	use gl_client::tls::TlsConfig;

	let tls = TlsConfig::new();
	```
	
=== "Python"

	```python
	from glclient import TlsConfig
	
	# Creating a new `TlsConfig` object will automatically load the dummy identity
	tls = TlsConfig()
	```
	

Finally, we can create the [`Signer`][signer] which processes incoming signature
requests, and is used when registering a node to prove ownership of
the private key. The last thing to decide is which network we want the
node to run on. You can chose between the following networks:

 - `testnet`
 - `bitcoin`

We'll pick `bitcoin`, because ... reckless 😉

=== "Rust"
	```rust
	use gl_client::signer::Signer;
	use gl_client::bitcoin::Network;
	let signer = Signer::new(secret, Network::Bitcoin, tls).unwrap();
	```

=== "Python"
	```python
	from glclient import Signer
	
	signer = Signer(seed, network="bitcoin", tls=tls)
	```
	
[bip39]: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki


## Registering a new node

Registering a node with the [`Scheduler`][scheduler] creates the node metadata on
the Greenlight service, including the node's identity and the public
key, and ensures everything is setup to start the node. 

In order to register a node, the client needs to prove it has access
to the corresponding private key. Since the private key is managed
exclusively by the [`Signer`][signer] we need to pass the [`Signer`][signer]
to the [`Scheduler`][scheduler]:

=== "Rust"
	```rust
	use gl_client::scheduler::Scheduler;
	use gl_client::bitcoin::Network;

	let scheduler = Scheduler::new(signer.node_id(), Network::Bitcoin).await.unwrap();

	// Passing in the signer is required because the client needs to prove
	// ownership of the `node_id`
	scheduler.register(&signer).await.unwrap();

	```

=== "Python"
	```python
	from glclient import Scheduler
	
	scheduler = Scheduler(
	    node_id=signer.node_id(),
		network="bitcoin",
		tls=tls,
	)
	
	# Passing in the signer is required because the client needs to prove
	# ownership of the `node_id`
	res = scheduler.register(signer)
	```

The result of `register` contains the credentials that can be used
going forward to talk to the scheduler and the node itself. 

!!! important 
	Please make sure to store them somewhere safe, since anyone with 
	these credentials can access your node.

=== "Rust"
	```
	let tls = TlsConfig::new().unwrap().identity(res.device_cert, res.device_key);
	
	// Use the configured `tls` instance when creating `Scheduler` and `Signer`
	// instance going forward
	let signer = Signer(seed, Network::Bitcoin, tls);
	let scheduler = Scheduler::with(signer.node_id(), Network::Bitcoin, "uri", &tls).await?;
	```

=== "Python"
	```python
	tls = TlsConfig().identity(res.device_cert, res.device_key)
	
	# Use the configured `tls` instance when creating `Scheduler` and `Signer`
	# instance going forward
	signer = Signer(seed, network="bitcoin", tls=tls)
	node = Scheduler(node_id=signer.node_id(), network="bitcoin", tls=tls).node()
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
