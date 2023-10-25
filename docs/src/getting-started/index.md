# Overview
<!-- Overview: What you'll get -->
<!-- Quick GL system intro -->

Before diving into the specifics, let's first define a number of
concepts that will be useful once we start developing:

## Nodes

Greenlight provisions and manages [Core Lightning][cln-github] nodes
on behalf of its users. The nodes expose the grpc interface defined in
the [`cln-grpc` proto file][cln-grpc-proto], without limitations. The
goal of this guide is to spin up a node and interact with it as if it
were a local Core Lightning node.

[cln-grpc-proto]: https://github.com/ElementsProject/lightning/blob/master/cln-grpc/proto/node.proto

## Authentication

All communication channels in Greenlight are authenticated and
encrypted via mTLS (mutual Transport Layer Security). Each client
receives its own identity in the form of a private key and matching
certificate, which can then be used to authenticate and encrypt
communication when talking with Greenlight.

This guide uses two types of identities

 - Developer identities can be used to register or recover Greenlight nodes.
 - Device identities are used by applications to authenticate to single Greenlight
   nodes. The private key must be generated locally and stay on your users
   device.

You can obtain a developer identity using the [Greenlight Developer
Console][gdc]. When you register a node Greenlight will return
a device identity. Alternatively, you can use request a new
device identity for a node using the recover functionality.

See the [security][sec] page for more details about how the
authentication works.

## Signer

The signer manages any private information, is used to prove node
ownership when registering and recovering, and processes signature
requests from the node. It is initialized with the secret seed (a 32
byte secret), the bitcoin network the node runs on, and the identity
to use when communicating with the node.

See the [security][sec] page for details on how the signer ensures
that operations it signs off originate from an authenticated app.

## Scheduler

Greenlight nodes are scheduled on-demand when a client needs to talk
to it. The Scheduler tracks which nodes are running where, and starts
them if they aren't running yet. You can think of it as just a
mechanism to register new nodes and look up where they are running.

<!-- Chose a language -->

[cln-github]: https://github.com/ElementsProject/lightning
[sec]: ../reference/security.md
[cert]: ./certs.md
[gdc]: https://greenlight.blockstream.com
