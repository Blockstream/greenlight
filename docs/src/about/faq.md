# Frequently Asked Questions

## Security

### Can the server cheat by broadcasting an old state? <a name="cheating-server"></a>

_Context_: Since the state of the node is stored on the server, could
an attacker use the server state to cheat, i.e., close a channel with
an outdated state?

**No, the signer needs to sign off**. Greenlight uses CLN under the
hood, and CLN defers signatures of the commitment transaction until
the channel is getting closed. This means that the server will indeed
have a stub of the commitment transaction, however it is missing the
signer's signature. When closing the channel, CLN will request the
signer to fill in the missing signature. Upon receiving the signature
request the signer:

 1. checks that the commitment corresponds to the latest state, i.e.,
    no old and revoked state is being signed.
 
 2. updates its internal state to remember that this channel is being
    closed, and it will never sign a newer commitment transaction
    going forward.
 
All of this ensures that only ever the latest state gets signed, and
that this signed state doesn't get revoked, making a cheat attempt
impossible.

## Connectivity

### Why can't I connect to the service from my school/work network?

For its authentication and authorization Greenlight uses mTLS (mutual
transport layer security), an extension on the usual TLS used for
secure communication in browsers. Unlike normal websites however,
Greenlight requires two things:

 - The server must reply with a server certificate signed by the Greenlight CA.
 - The client must use a client certificate signed by the Greenlight CA.
 
When you try to access a service that uses mTLS (Mutual Transport
Layer Security) with self-signed certificates, you might encounter
connectivity issues, especially on networks with Deep Packet
Inspection (DPI).

DPI is a network security technique used to inspect network traffic to
identify potential threats. Some DPI systems can interfere with
encrypted connections, particularly those using self-signed
certificates. These systems often rely on trusted Certificate
Authorities (CAs) to validate certificates. Since self-signed
certificates are not issued by a trusted CA, they may be flagged as
suspicious and blocked.

The root cause of the issue lies in the network configuration and
security policies of your school or workplace network. They may have
strict security measures in place that restrict traffic based on
certificate validation.  ￼

This is not a Greenlight issue. Greenlight is using a standard
security protocol, mTLS, to protect your data. The problem arises from
the network restrictions imposed by your institution.

We are working on exposing the scheduler and node interfaces over
[`grpc-web`][grpc-web] which can use browser-grade certificates, and
not require a client certificate, thus avoiding these connectivity
issues.

[grpc-web]: https://github.com/grpc/grpc-web
