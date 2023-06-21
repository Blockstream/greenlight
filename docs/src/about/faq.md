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
