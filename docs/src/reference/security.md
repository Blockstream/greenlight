# Security

<!--
Overview: draw diagrams of how requests flow in this system

Who is authenticating to whom?
-->

## Client &rlarr; Node authentication

<!--
Describe the mTLS authentication for the transport.
Structure of the CA, confining each user into their own space

Outlook for `runes` and how they can be used to limit actions
-->


## Client &rlarr; Signer authentication

<!--
Document how we use a signature from the signer to authorize a client key to sign payloads.

Document how those signatures are passed to the node and signer, and how the signer verifies its authenticity, and authorizes the command

Describe how this prevents attackers from injecting commands directly at the node.
-->
