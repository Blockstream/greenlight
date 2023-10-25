# Recovering access to a node

One of the core benefits of Greenlight over self-hosted nodes is the
ability to separate the state management in the form of the node
database from the authorizing party, i.e., the [`Signer`][signer]. This allows
users to seamlessly recover access to their node in case of a boating
accident, by simply proving ownership of the private key that
corresponds to that node.

In order to recover access all you need to do is recover the `seed` from the BIP39 seed phrase and initialize the [`Signer`][signer] with it:

=== "Rust"
	```rust
	use gl_client::{Signer, TlsConfig, Scheduler, Bitcoin};
	
	let cert = ...; // Your developer certificate (client.crt)
	let key = ...; // Your developer key (client-key.pem)

	let tls = TlsConfig().identity(cert, key);
	let signer = Signer(seed, Network::Bitcoin, tls);
	let scheduler = Scheduler::new(signer.node_id(), Network::Bitcoin).await;
	
	let res = scheduler.recover(&signer).await?;
	```
	
=== "Python"
	```python
	from glclient import Scheduler, Signer, TlsConfig
	
	cert = ... // Your developer certificate
	key = ... // Your developer key

	tls = TlsConfig().identity(cert, key);
	signer = Signer(seed, network="bitcoin", tls=tls)
	scheduler = Scheduler(
	    node_id=signer.node_id(),
		network="bitcoin",
		tls=tls,
	)
	res = scheduler.recover(signer)
	```

Notice that we are using a `TlsConfig` that is not configured with a
client certificate and key, because that's what we're trying to
recover. In the background the `scheduler` instance will contact the
[`Scheduler`][scheduler] service, retrieve a challenge, sign that challenge with the
signer, and then call `recover` with that proof signature. The
Scheduler will then check the challenge-response passed to `recover`
and if successful, generate a new certificate for the client, which
will provide access to the node.

!!! important
	Remember to store the `device_cert` and `device_key` from the result
	so you can load it next time you want to interact with the node.

[signer]: ./index.md#signer
[scheduler]: ./index.md#scheduler
