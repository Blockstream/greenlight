# Client Credentials

Greenlight utilizes various components, such as mTLS certificates and 
runes, to provide secure access to a node. 
Various pieces of data must be persisted and made available to the client 
at runtime. Check out the [Security][security] page for more information.

To simplify data management for users and developers, Greenlight uses 
`Credentials` as a centralized location for the necessary information. 
Credentials can be reconstructed in various ways and exported in 
byte-encoded format for persistence.

!!! tip
	If you registered your greenlight node before the release of gl-client v0.2, please see the section [Instantiating a Credential from device certificates] below.

## The two variants of the Credential type

There are two variants of the Credential type, the Nobody credential and the Device credential.

### Nobody Identity

The Nobody credential is the credential that's used when there is no registered node associated with your request. It can be initialized using the developer certificates acquired from the [Greenlight Developer Console][gdc] and is used for registering and recovering greenlight nodes.

[gdc]: https://greenlight.blockstream.com

A reference instantiation taken from the [examples](https://github.com/Blockstream/greenlight/tree/main/examples/rust) can be found below:

=== "Rust"
	```rust
--8<-- "main.rs:dev_creds"
	```
	
=== "Python"
	```python
--8<-- "main.py:dev_creds"
	```

### Device Identity

The `Credentials` for a device can be retrieved in numberous ways. They can be restored from a path to a encoded credentials file, as well as from a byte array that carries the same data. `Credentials` for the device can also be constructed by their components or a combination of all of the above.

How to build `Credentials` from encoded formats?

=== "Rust"
	```rust
--8<-- "main.rs:device_creds"
	```

=== "Python"
	```python
--8<-- "main.py:device_creds"
	```

## Instantiating a Credential from device certificates



[security]: ./security.md