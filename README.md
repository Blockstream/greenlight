# Blockstream Greenlight

[![Read the Documentation](https://img.shields.io/badge/Read-Documentation-blue)](https://blockstream.github.io/greenlight/)
[![Crates.io](https://img.shields.io/crates/d/gl-client)](https://crates.io/crates/gl-client)

This repository contains everything to get you started with
Blockstream Greenlight, your self-sovereign Lightning node in the
cloud.

greenlight exposes a number of services over grpc allowing
applications to integrate, and users to manage and control their node
running on our infrastructure. The [protocol buffers files][protos]
are provided as well as a number of language bindings for easier
integration.

The core two services currently exposed are:

 - The __scheduler__ which allows users and applications to register
   new accounts, recover accounts based on the seed secret used to
   create the account, schedule their node on our infrastructure and
   looking up already scheduled nodes. Scheduling returns a grpc URI
   where the node itself can be reached.
 - The __node__ represents the scheduled user node running, and is
   used to interact with the c-lightning instance. It can be used to
   send and receive payments, manage channels and liquidity.

The application can have two possible roles:

 - __Remote control__: the application can authenticate as a user and
   can interact with its node to receive payments, initiate payments,
   manage channels and funds.
 - __Key manager__: the application has access to the secret keys that
   are necessary to sign off on actions initiated by a remote control,
   or as a reaction to some state change on the node. This usually
   involves running a part of c-lightning called the `hsmd` and is the
   binary portion of the language bindings in this repository.
   
An application can implement either one or both of these roles at the
same time. Particular care has to be taken when implementing the key
manager role, but only one application implementing this role must be
present at a time, freeing others from that duty.

# Getting started

The easiest way to begin using __Greenlight__ is with its command-line
interface `glcli`. You can install it directly from __crates.io__ by running:
```bash
cargo install gl-cli
```

Once installed execute:
```bash
glcli --help
```
This command will display an overview of all available commands.

For additional details and usage examples, refer to `glcli` [README][glcli-doc].

[glcli-doc]: libs/gl-cli

# Best practices

## Secret generation

The language bindings expect a 32 byte securely generated secret from
which all private keys and secrets are generated. This secret must be
kept safe on the user device, and under no circumstances should be
stored on the application server, as it controls the user funds. When
generating the seed secret ensure that the source of randomness is
suitable for cryptographically secure random numbers!

In order to guarantee portability, the seed should be generated
according with the [BIP 39][bip39] standard, and show the mnemonic
during the creation so they can initialize other client applications
with the same secret. The mnemonic should not be shown afterwards.

## Network

greenlight currently supports 3 networks: `bitcoin`, `testnet` and
`regtest`. We suggest mostly using `testnet` for testing. We plan to
open up our `regtest` and add `signet` in the near future to make
testing simpler as well, but the public testnet should serve that
purpose well for now. Keep in mind that the `testnet` can sometimes be
a bit flaky, and the lightning network running on testnet is not the
best maintained, expect `bitcoin` to perform better 🙂

## Preemption

Scheduled nodes are preempted after some minutes of inactivity. The
timer can be reset by performing any interaction, except attaching the
key device. This is to conserve server resources, reflect that the
node can't do much without a key manager attached, and provide our
operational team the flexibility to take down nodes for
maintenance. There is currently no absolute deadline by which nodes
are shut down, however keep in mind that that might eventually become
necessary if applications just keep nodes alive indefinitely.

## Latencies

Currently the environment consists of a single cluster in `us-west2`,
with both scheduler and nodes in this region. We plan to implement
geo-load-balancing of the nodes (and associated databases) and thus
considerably reduce the roundtrip times from the rest of the world.

Currently the roundtrip times can be relatively high from more distant
regions, and an mTLS handshake requires multiple roundtrips the first
time (parts of the handshake can be cached in memory and skipped on
reconnects). This is only temporary until geo-load-balancing gets
rolled out.

To minimize the overhead of the mTLS handshake it is suggested to keep
the grpc connections open and reuse them whenever possible.

# Changelog



[bip39]: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
[protos]: https://github.com/Blockstream/greenlight/blob/main/libs/proto/
.
