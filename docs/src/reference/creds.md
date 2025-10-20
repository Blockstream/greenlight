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
	If you registered your greenlight node before the release of gl-client v0.2, please see the section [Instantiating a Credential from device certificates](#instantiating-a-credential-from-device-certificates) below.

## Credential Types
There are two types of Credentials, the Nobody credential and the Device credential.

Reference instantiations of both credentials types can be found below. Complete files can be viewed from the [examples](https://github.com/Blockstream/greenlight/tree/main/examples/rust) folder from which these code snippets were taken.

### Nobody Credentials

The Nobody credentials are used when there is no registered node associated with the requests made with the credential. They can be initialized using the developer certificates acquired from the [Greenlight Developer Console][gdc] and are used for registering and recovering greenlight nodes.

[gdc]: https://greenlight.blockstream.com

=== "Rust"
	```rust
--8<-- "getting_started.rs:dev_creds"
	```
	
=== "Python"
	```python
--8<-- "getting_started.py:dev_creds"
	```

### Device Credentials

The Device credentials are used when there is a registered node associated with the requests made with the credential. They can be restored from a path to a encoded credentials file, as well as from a byte array that carries the same data. `Credentials` for the device can also be constructed by their components or a combination of all of the above.

=== "Rust"
	```rust
--8<-- "getting_started.rs:device_creds"
	```

=== "Python"
	```python
--8<-- "getting_started.py:device_creds"
	```

## Instantiating a Credential from device certificates

For glclient versions released before v0.2, device certificates were the primary mechanism used for authentication. These certificates can be upgraded by instantiating a Device credential and invoking the upgrade method with a Nobody-instantiated Scheduler and a Signer. This will give the upgrade method everything it needs to construct any missing details and will return a properly functioning Device credential.

=== "Rust"
	```rust
--8<-- "getting_started.rs:upgrade_device_certs_to_creds"
	```

=== "Python"
	```python
--8<-- "getting_started.py:upgrade_device_certs_to_creds"
	```


[security]: ./security.md
