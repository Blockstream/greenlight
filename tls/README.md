# mTLS Certficates and anonymous Key

This directory contains the mTLS certificate and key for the anonymous
user, and the root CA certificate all other certificates are rooted
in. The anonymous `/users/nobody` key is necessary since the servers
will always enforce mutual TLS authentication, even on first
contact. Notice that the only system that'll accept a connection using
this key is the scheduler. In order to talk to any other service you
will need to initialize the client library with the user-specific
keypair that you get by registering or recovering a node on
greenlight.
