# Blockstream Greenlight

This repository contains everything to get you started with
Blockstream Greenlight, your self-sovereign Lightning node in the
cloud.

> 🚧 While this repository is currently set to private it'll be set to
> public once we open up to the wider public, so please don't share
> internals that aren't supposed to be public. 🚧

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

The following is a quick walkthrough based on the python `glcli`
command line tool to get you started:

## Install and updating `glcli` and python API

There are prebuilt `glcli` and `gl-client-py` packages on a private
repository. These allow developers to hit a running start, without
having to bother with compiling the binary extensions.

```bash
pip install --extra-index-url=https://us-west2-python.pkg.dev/c-lightning/greenlight-pypi/simple/ -U glcli
```

Should you encounter any issues with the installation it is likely due
to there not being a prebuilt version of the `gl-client-py`
library. Please refer to its [documentation][glpy-doc] on how to build
the library from source, and let us know your platform so we can add
it to our build system if possible.

[glpy-doc]: libs/gl-client-py

## Register / recover an account

Registration and recovery are managed by the scheduler, hence the
`scheduler` prefix in the following commands.

```
$ glcli scheduler register --network=testnet
{
  "device_cert": "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----\n\n\n",
  "device_key": "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n"
}
```

This returns an mTLS certificate and a matching private key that is
used to authenticate and authorize the application with the
services. These should be stored on the device and be used for all
future communication. In particular, nodes will only accept incoming
connections that are authenticated with the user's certificate. In
order to register as a new user a signature from the key manager is
required.

The recovery process is also based on the key manager providing a
signature:

```bash 
$ glcli scheduler recover
{
  "device_cert": "-----BEGIN CERTIFICATE-----\n...\n-----END CERTIFICATE-----\n\n\n",
  "device_key": "-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n"
}
```

This too provides a certificate and a matching private key that can be
used to authenticate and authorize the application.

## Scheduling

While `glcli` takes care of scheduling the node automatically if
another command is provided, when implementing the client this must be
done as a separate step:

```bash
$ glcli scheduler schedule
{
  "grpc_uri": "https://35.236.110.178:6019",
  "node_id": "A27DtykCS7EjvnlUCB0yjrSMz4KSN4kGOo0Hm2Gd+lbi"
}
```

Notice that protocol buffers encode binary values using base64, which
is why the `node_id` isn't hex encoded. The node can now be reached
directly at the provided URI. Notice that `glcli` will automatically
look up the current location:

```
$ glcli getinfo
{
  "addresses": [],
  "alias": "LATENTGLEE",
  "blockheight": 2003446,
  "color": "A27D",
  "network": "testnet",
  "node_id": "A27DtykCS7EjvnlUCB0yjrSMz4KSN4kGOo0Hm2Gd+lbi",
  "num_peers": 0,
  "version": "0.10.0"
}
```

In order to attach the `hsmd` to the node run the following:

```bash
$ glcli hsmd 
[2021-06-07 18:38:02,574 - DEBUG] Found existing hsm_secret file, loading secret from it
[2021-06-07 18:38:02,575 - DEBUG] Initializing libhsmd with secret
[2021-06-07 18:38:02,583 - DEBUG] libhsmd initialized for node_id=036ec3b729024bb123be7954081d328eb48ccf82923789063a8d079b619dfa56e2
[2021-06-07 18:38:02,584 - DEBUG] Contacting scheduler at 35.236.110.178:2601 to wait for the node to be scheduled.
[2021-06-07 18:38:02,594 - DEBUG] Waiting for node 036ec3b729024bb123be7954081d328eb48ccf82923789063a8d079b619dfa56e2 to be scheduled
[2021-06-07 18:38:03,229 - INFO] Node was scheduled at https://35.236.110.178:6019, opening direct connection
[2021-06-07 18:38:03,230 - DEBUG] Streaming HSM requests
```

Not all commands require the `hsmd` to be running, however it is good
practice to have it running in parallel with other commands being
executed. Future versions of `glcli` will automatically spawn an
instance if needed by the command in question.

From hereon the node can be managed just as if it were a local node,
including sending and receiving on-chain transactions, sending and
receiving off-chain transactions, opening and closing channels, etc.

```bash

glcli --help
Usage: glcli [OPTIONS] COMMAND [ARGS]...

Options:
  --help  Show this message and exit.

Commands:
  close
  connect
  destroy
  disconnect
  fundchannel
  getinfo
  hsmd         Run the hsmd against the scheduler.
  invoice
  listfunds
  listpeers
  newaddr
  pay
  scheduler
  stop
  withdraw
```

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
