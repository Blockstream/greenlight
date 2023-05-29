# Partner Certificates

Greenlight includes an _invite system_ that is used to control the
influx of new users, and avoid overloading the system without
sufficient time to scale up the infrastructure. In order to register a
new node the client has to present an invite code when calling
`Scheduler.register()`. The invite code can only be used once, and the
plan is to regularly top up invites existing users can hand out.

In order not to curtail your product's growth through these
invitations, Greenlight also includes a bypass system through the
**partner certificates**. These are custom certificates that partners
can bundle with their application, and that allow registering new
nodes without having to rely on invites.


!!! info "Impact of registration method on the node"
	Independently of whether you registered your node with a partner
	certificate or an invite code, you'll get the same service. Nodes can
	be used from a multitude of different applications, independently of
	these applications using one mechanism or the other. Nodes that
	predate the introduction of the invite system.

## Who gets a Partner Certificate?

As a rule of thumb, once your application has over 1'000 users, each
with their own node, you'll want to ask us for a partner
certificate. Upon contacting us we will verify that your application
is suitable for a partnership with us, and provide you with the
certificate to use.

Suitability is dependent both on the application, as well as the
overall status of Greenlight. Since this is a surge-protection
mechanism for Greenlight, we cannot accept all applications, but we do
our best.

Once you have the certificate you can follow the instructions below.

## Using a Partner Certificate

The partner certificate is a custom version of the `/users/nobody`
certificate that is used to bootstrap the trust chain for clients that
have not yet received their own private key and certificate specific
to their node. As such the private key and certificate are compiled
into `gl-client` at build time.

The certificate is ditributed as two `x509` PEM files bundled into an
encrypted zip file:

 - `partner-{NAME}.pem`: this is the certificate that the client will
   present when connecting to the Scheduler in order to either
   `register()` or `recover()`.
 - `partner-{NAME}-key.pem`: this is the private key matching the
   above certificate and is used to encrypt the transport and
   authenticate as a partner to the Scheduler, allowing to bypass the
   invite code requirement.

Alongside the encrypted zip file, you will also receive the password
to decrypt the zip file.

Ideally the two files are then stored, securely, alongside the code,
in encrypted form, or instrument your CI system to have access to them
when building. Treat the private key with the same care you'd use for
an API key, as they fulfill identical roles in this scenario.

In order to tell the build to use the partner certificate you'll have
to set two environment variables in the build environment. Please
consult your build system and/or shell environment about how to ensure
teh build sees the variables.

 - `GL_CUSTOM_NOBODY_KEY` should have the absolute path to `partner-{NAME}-key.pem`
 - `GL_CUSTOM_NOBODY_CERT` should have the absolute path to `partner-{NAME}.pem`
 
If either of these is not set you'll get a warning that this is an
invite-based client:

```
warning: Using default NOBODY cert.
warning: Set "GL_CUSTOM_NOBODY_KEY" and "GL_CUSTOM_NOBODY_CERT" to use a custom cert.
```

## When (not) to use the partner certificate

In order to retain the protection aspect of the partner certificates
please only use them in your own applications, and don't share them
with others, directly or indirectly. In particular this means that you
should **not include** them if you are building a library that others
will use.
