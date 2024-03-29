# Using the Certificate

In order to build with Greenlight, you need a certificate.
These are custom certificates that developers can bundle with their application,
and that allow registering new nodes.

## How to get a Certificate?

Create an account on the [Greenlight Developer Console][gdc] and download the zip file
containing the certificate.

[gdc]: https://greenlight.blockstream.com

## Using the Certificate

The certificate is a custom version of the `/users/nobody`
certificate that is used to bootstrap the trust chain for clients that
have not yet received their own private key and certificate specific
to their node. As such the private key and certificate are compiled
into `gl-client` at build time.

The certificate is ditributed as two `x509` PEM files bundled into a zip file:

 - `client.crt`: this is the certificate that the client will
   present when connecting to the Scheduler in order to either
   `register()` or `recover()`.
 - `client-key.pem`: this is the private key matching the
   above certificate and is used to encrypt the transport and
   authenticate as a partner to the Scheduler.

Ideally the two files are then stored, securely, alongside the code,
in encrypted form, or instrument your CI system to have access to them
when building. Treat the private key with the same care you'd use for
an API key, as they fulfill identical roles in this scenario.

In order to tell the build to use the certificate you'll have
to set two environment variables in the build environment. Please
consult your build system and/or shell environment about how to ensure
the build sees the variables.

 - `GL_CUSTOM_NOBODY_KEY` should have the absolute path to `client-key.pem`
 - `GL_CUSTOM_NOBODY_CERT` should have the absolute path to `client.crt`

If either of these is not set you'll get a warning. This warning
can be ignored if you are using an invite-code.

```
warning: Using default NOBODY cert.
warning: Set "GL_CUSTOM_NOBODY_KEY" and "GL_CUSTOM_NOBODY_CERT" to use a custom cert.
```

## Providing the certificates at runtime

In case you do not want to provide the certificate at compile-time,
e.g., because you are using pre-compiled language bindings, you can
also provide the certificates at runtime. The following code snippets show how to construct a `Signer` and a `Scheduler` instance with the certificates:

=== "Rust"
	```rust
	use gl_client::tls::{Signer, Scheduler, TlsConfig};
	let tls = TlsConfig::new()?.identity(certificate, key);

	let signer = Signer(seed, Network::Bitcoin, tls);

	let scheduler = Scheduler::with(signer.node_id(), Network::Bitcoin, "uri", &tls).await?;
	```

=== "Python"
	```python
	from glclient import TlsConfig, Signer, Scheduler
	tls = TlsConfig().identity(res.device_cert, res.device_key)

	signer = Signer(seed, network="bitcoin", tls=tls)

	node = Scheduler(node_id=signer.node_id(), network="bitcoin", tls=tls).node()
	```

Notice that this is the same way that the `TlsConfig` is configured
with the user credentials provided from the `register()` and
`recover()` results.

!!! important
	Certificates are credentials authenticating you as the
	developer of the Application, just like API keys. Do not publish
	the keys, as that would allow others to impersonate you.

As for where the certificate may be stored, please use a location that
is not easily accessible by users. Alternatively you can also provide
them via from your servers gated behind an additional authentication
layer.

## When (not) to use the certificate

In order to retain the protection aspect of the certificates
please only use them in your own applications, and don't share them
with others, directly or indirectly. In particular this means that you
should **not include** them if you are building a library that others
will use.
