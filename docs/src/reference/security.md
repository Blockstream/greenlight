# Security

Each component in the Greenlight system is uniquely identified by an
mTLS keypair, also called an _identity_. These are either used
directly to setup mutually authenticated TLS connections, or to sign
payloads in cases direct connections are not desirable or possible.

In addition the signer also has access to the Bitcoin keypair that
backs the Node ID, as well as the on-chain wallet. We will refer to
this keypair as the _signer-identity_, whereas mTLS keypairs are just
called the _client-identities_.

In the following scenarios we will consider an attacker that has
access to the node infrastructure, but not the client or the
signer. This can either be an external attacker or a rogue Greenlight
operator. Our goal is to prevent any access to funds from such an
attacker, whether internal or external, by checking authorization on
both the node as well as the signer level.


## Client &rlarr; Node Authentication

This is a direct connection from a client to the node, or the signer
to the node. The mTLS certificate hierarchy is under the control of
the Greenlight Team. Each user gets their own CA, and nodes are
configured to only accept client connections from certificates
matching that CA. This guarantees that users can only contact their
own node, while all other nodes would cause a mismatch and disconnect
the client.

!!! example "Experimental"
	Access to the node is currently all-or-nothing. The planned
	introduction of Rune-based access control will enable users
	to limit the operations a given client can execute. This is
	dependent on the _pairing process_ as any client that has
	access to the signer could just escalate its privileges via
	the recovery.

The private key for the client-identity is generated on the client
itself, and never leaves the client, sending only a certificate
signature request (CSR) to the scheduler which creates and signs the
certificate. This puts the client in the correct subtree of the CA
hierarchy, enabling it to contact the node.

Impersonation by a potential attacker is prevented by keeping the
private key for the client-identity on the client, and not share it
with the server. Notice however that the Greenlight team, being in
control of the CA hierarchy, could create a bogus client certificate
and use that to issue commands to the node. More on this in the next
section.

## Client &rlarr; Signer Authentication

The signer cannot rely on the mTLS CA structure, since that is under
control of the Greenlight team. Instead, Greenlight employs an 
attestation scheme in which the signer identity attests to itself and 
other signers that a particular client is authorized to perform 
certain operations.

When registering a new client, the signer will submit a [`rune`][rune]
as an access token. The rune serves mainly as an authentication while the 
client's identity in form of a mTLS key-pair provides authorization. 
The run is bound to the client's identity; it is presented to the signer 
on each request. This will allow multiple signers to recognize which 
clients are authorized, even if the signer who carved the rune is not the 
signer verifying the authorization.

Before signing a request the signer independently verifies that:

 1. The operations that it is asked to sign off on match pending RPC
    commands, and are safe to perform.
 2. The pending RPC commands are all signed by a valid client-identity
 3. The client-identity has a valid rune that qualifies for the request.
 4. None of the pending RPC commands is a replay of a previously
    completed RPC command.

An attacker that has gotten access to the node infrastructure may
inject RPC commands directly into the node, side-stepping any
authorization check on the node. For this reason the signer performs
the same checks both on the node as well as the signer, the former
preventing read-access that doesn't involve the signer, while the
latter ensures funds are not moved without a client authorizing it.

The client-identity pubkey, its signature of the command payload, and
the rune are all passed to the node via grpc headers. The node 
extracts them, alongside the call itself, and adds it to a request 
context which will itself be attached to requests that are sent to the
signer, so it can verify the validity and authenticity of the 
operations. An attacker that gains access to the node is unable to 
provide either these signatures and will therefore fail to convince
the signer of its injected commands.

## Client &rlarr; Signer Authorization

The [`rune`][rune]-based signer authentication verifies a client's 
identity and their authorization to execute commands. To achieve more 
granular control over command authorization for individual clients, a 
rune can be created with specific restrictions. When verifying client 
identity, the signer additionally ensures that specific conditions, 
such as the requested command, align with the limitations of the given
rune.

This enables a user to share access to the signer across multiple 
clients that are restricted to a subset of commands, for example 
read-only clients, or invoice-only clients, or clients that can only
create on-chain addresses and so on.

<!-- FIXME: Should I point to the library that we use here (which is futhark, that has a link to rusty's library)? Maybe we should give runes a Wikipedia article ^^ -->
[rune]: https://github.com/nepet/runeauth
