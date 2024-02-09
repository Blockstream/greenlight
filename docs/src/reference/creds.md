# Client Credentials

Greenlight utilizes various components, such as mTLS certificates and 
runes, to provide secure access to a node. 
Various pieces of data must be persisted and made available to the client 
at runtime. Check out the [Security][security] page for more information.

To simplify data management for users and developers, Greenlight uses 
`Credentials` as a centralized location for the necessary information. 
Credentials can be reconstructed in various ways and exported in 
byte-encoded format for persistence.

## Ways to Retrieve Credentials

### Nobody Identity

How to build Credentials for the *default* `Nobody` identity?

=== "Rust"
	```rust
	use gl_client::credentials::Builder;

    // Builds the default nobody identity.
	let creds = Builder::as_nobody()
		.with_default()
		.expect("Failed to create default Nobody credentials")
		.build()
		.expect("Failed to build Nobody credentials");

    // Access the tls config.
    let tls_config = creds.tls_config()
		.expect("Failed to create TlsConfig");
	```

=== "Python"
	```python
	from glclient import Credentials, TlsConfig

	# Builds the default nobody identity.
	creds = Credentials.as_nobody().with_default().build();
	
	# Access the TlsConfig
	tls = TlsConfig(creds)
	```

How to build Credentials for a *custom* `Nobody` identity?

=== "Rust"
	```rust
	use gl_client::credentials::Builder;

	let ca = std::fs::read("ca.pem").expect("Failed to read from file");
	let cert = std::fs::read("nobody.pem").expect("Failed to read from file");
	let key = std::fs::read("nobody-key.pem").expect("Failed to read from file");

    // Builds nobody credentials from custom values.
	let creds = Builder::as_nobody()
		.with_ca(ca)
		.with_identity(cert, key)
		.build()
		.expect("Failed to build Nobody credentials");

    // Access the tls config.
    let tls_config: gl_client::tls::TlsConfig = creds.tls_config()
		.expect("Failed to create TlsConfig");
	```

=== "Python"
	```python
	from pathlib import Path
	from glclient import Credentials, TlsConfig

	capath = Path("ca.pem")
	certpath = Path("nobody.pem")
	keypath = Path("nobody-key.pem")

	# Builds the default nobody identity.
	creds = Credentials.as_nobody()
		.with_ca(capath.open(mode="rb").read())
		.with_identity(
			certpath.open(mode="rb").read(),
			keypath.open(mode="rb").read(),
		)
		.build()
	
	# Access the TlsConfig
	tls = TlsConfig(creds)
	```

### Device Identity

The `Credentials` for a device can be retrieved in numberous ways. They can be restored from a path to a encoded credentials file, as well as from a byte array that carries the same data. `Credentials` for the device can also be constructed by their components or a combination of all of the above.

How to build `Credentials` from encoded formats?

=== "Rust"
	```rust
	use gl_client::credentials::Builder;

    // Restore device credentials from a file.
	let creds = Builder::as_device()
		.from_path("path/to/credentials/file")
		.expect("Failed to read credentials file")
		.build()
		.expect("Failed to build Device credentials");


	// Alternatively restore from byte encoded data;
	let enc_creds: [u8] = vec![...] // Some useful data here.
	let creds = Builder::as_device()
		.from_bytes(&enc_creds)
		.expect("Faild to decode credentials")
		.build()
		.expect("Failed to build Device credentials");

    // Access the tls config.
    let tls_config = creds.tls_config()
		.expect("Failed to create TlsConfig");

	// Access the rune.
	let rune = creds.rune();
	```

=== "Python"
	```python
	from glclient import Credentials, TlsConfig

	# Restore device credentials from a file.
	creds = Credentials.as_device()
		.from_path("/path/to/credentials/file")
		.build()

	# Alternatively restore from byte encoded data.
	creds = Credentials.as_device()
		.from_bytes(b('...')) # Some meaningful data
		.build()
	
	# Access the TlsConfig
	tls = TlsConfig(creds)
	```

How to build `Credentials` step by step?

=== "Rust"
	```rust
	use gl_client::credentials::Builder;

	let ca = std::fs::read("ca.pem").expect("Failed to read from file");
	let cert = std::fs::read("device.pem").expect("Failed to read from file");
	let key = std::fs::read("device-key.pem").expect("Failed to read from file");
	let rune = std::fs::read("rune").expect("Failed to read from file");

    // Build device credentials step by step.
	let creds = Builder::as_device()
		.with_ca(ca)
		.with_identity(cert, key)
		.with_rune(rune)
		.build()
		.expect("Failed to build Device credentials");

    // Access the tls config.
    let tls_config = creds.tls_config()
		.expect("Failed to create TlsConfig");

	// Access the rune.
	let rune = creds.rune();
	```

=== "Python"
	```python
	from pathlib import Path
	from glclient import Credentials, TlsConfig

	capath = Path("ca.pem")
	certpath = Path("device.pem")
	keypath = Path("device-key.pem")
	runepath = Path("rune")

	# Builds the default nobody identity.
	creds = Credentials.as_nobody()
		.with_ca(capath.open(mode="rb").read())
		.with_identity(
			certpath.open(mode="rb").read(),
			keypath.open(mode="rb").read(),
		)
		.with_rune(runepath.open(mode="r").read())
		.build()
	
	# Access the TlsConfig
	tls = TlsConfig(creds)
	```

!!! tip
	One can use a combination of the methods showed above to override the configuration. This example overrides the CA certificate.

	=== "Rust"
		```rust
			use gl_client::credentials::Builder;

			let ca = std::fs::read("ca.pem").expect("Failed to read from file");

			// Builds the default nobody identity, overrides ca certificate.
			let creds = Builder::as_nobody()
				.with_default()
				.expect("Failed to create default Nobody credentials")
				.with_ca(ca) // CA certificate gets overriden.
				.build()
				.expect("Failed to build Nobody credentials");
		```
	
	=== "Python"
		```python
		from glclient import Credentials, TlsConfig

		capath = Path("ca.pem")

		# Builds the default nobody identity, overrides ca certificate.
		creds = Credentials
			.as_nobody()
			.with_default()
			.with_ca(capath.open(mode="rb").read()) # CA certificate gets overriden.
			.build();
		
		# Access the TlsConfig
		tls = TlsConfig(creds)
		```


[security]: ./security.md