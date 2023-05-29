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
encrypted via mTLS (mutual Transport Layer Security), similar to HTTPS
in the browser. Each client receives its own identity in the form of a
private key and matching certificate, which can then be used to
authenticate and encrypt communication when talking with Greenlight.

In this guide we will be using two identities:

 - `nobody`: An identity that is shipped with the library, used to
   communicate with services that don't require authentication.
 - A Device Identity: A unique identity used by the application to
   authenticate with the Greenlight. The private key is generated and
   kept locally, and certified by Greenlight to belong to a given
   node.

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

## Invite Tokens

As we gradually spin up and extend our services during the early phase
of the project, our resources are limited. You will need a 
[partner certificate][partner-cert] or an invite code to register a 
node at the Greenlight service. You can request an invite code via our
[invite-form][invite-tokens]. Please note that invite codes are 
limited.

<!-- Chose a language -->

[cln-github]: https://github.com/ElementsProject/lightning
[sec]: ../reference/security.md
[partner-cert]: ../reference/partner-certs.md
[invite-tokens]: https://docs.google.com/forms/d/e/1FAIpQLSf_YaUJt8lKIDwS893Uk2mBiW6BUcoQkvO_g8EFZc9XqQfkqw/viewform