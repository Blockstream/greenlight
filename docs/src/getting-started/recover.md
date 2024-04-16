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
--8<-- "main.rs:recover_node"
	```
	
=== "Python"
	```python
--8<-- "main.py:recover_node"
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
