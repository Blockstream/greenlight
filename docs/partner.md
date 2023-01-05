# Partner Docs

This document is aimed at Greenlight partners that integrate Greenlight nodes into their services.

## The NOBODY Certificate

All connections to Greenlight and Nodes running on Greenlight are TLS encrypted. When a new Node is registered at Greenlight, we sign a SSL certificate that is used to both encrypt all future connection as well as to authenticate the client at the Greenlight services via mutual TLS authentication (mTLS).

On first contact the client uses a dummy certificate that is used to create an encrypted connection to the Greenlight _Scheduler_ that handles the registration process. The dummy certificate can only authenticate a client at the _Scheduler_ services.

The Greenlight client comes with a built-in default dummy certificate. However, a client can only register a node with the default dummy certificate if has access to a valid, unused _Invite-Code_ that is must be presented upon registration.

To avoid the need of an invitation, an integration partner can request a custom dummy certificate from Greenlight that can then be baked into the client that the partner delivers. With this registered custom dummy certificate there is no need to show a valid invite code to Greenlight upon registration of the partners clients nodes.

A partner that has a dummy certificate can bake it into a custom client on build time by setting the following environmental variables pointing to the dummy certificate file and the respective dummy key file:

`GL_CUSTOM_NOBODY_CERT=...`
`GL_CUSTOM_NOBODY_KEY=...`

The `gl-client` can then be build following the standard guide, using `cargo build` and will include the custom partner certificates into the source code. 
___Attention__: This leads to shasums of the partner client that do not match with the clients built by Greenlight using the default dummy certificate._