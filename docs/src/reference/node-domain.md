# Node Domain

The Greenlight team is committed to providing users with full control
over their Lightning nodes, which is why the ability to offboard from
a hosted non-custodial node to a self-hosted setup is a key
feature. We recognize that users may start with a hosted solution for
convenience, but as their needs grow, they might prefer to manage
their node on their own hardware or infrastructure. To facilitate this
transition, we have implemented an export functionality that allows
users to seamlessly move their node off our infrastructure. This
feature disables the hosted node and provides users with an encrypted
copy of their node's database. With this export, users can deploy
their node independently, ensuring they retain control and ownership
of their Lightning Network experience.

Nodes can be exported entirely, including all of their funds and
channel remaining intact. This means that apart from a brief downtime
while the node is being exported and re-deployed, the hurdle to taking
ownership of your own node is minimal.

Users who wish to run a Lightning Network node behind a NAT (Network
Address Translation) or firewall often face significant connectivity
issues. The primary challenge is that NATs and firewalls typically
block incoming connections, which prevents clients (such as wallets or
other nodes) from being able to connect to the node. This lack of
inbound connectivity is crucial for a non-custodial hosted Lightning
node, since the user cannot access the node via their app, without
configuring the network router or firewall.

This is why we built the node domain server, a reverse proxy that
smooths the user experience and hides the connection details behind a
simple, publicly reachable URL.

## Enter the _Node Domain_

The _node domain_ is a unique URL designed to provide seamless access
to a user's Lightning node, regardless of its location — whether
hosted on Greenlight's infrastructure or self-hosted by the user. The
format of the URL is `gl1[node_id].gl.blckstrm.com`, where `[node_id]`
is the `bech32m`-encoded Node ID for the user's node. This URL serves
as a fixed endpoint that users and clients can rely on to connect to
their node without needing to worry about the underlying network
configuration or location.

The functionality of the node domain is powered by a reverse
proxy. When a connection request is made to the node domain, the
reverse proxy processes the request and identifies the desired
endpoint using Server Name Indication (SNI). SNI allows the proxy to
determine which node the connection is intended for without decrypting
the data stream, ensuring that the connection remains secure.

Once  the  endpoint is  identified,  the  reverse proxy  forwards  the
connection to the appropriate node:

 - Hosted Nodes: If the node is hosted on Greenlight's infrastructure,
   the proxy checks if the node is currently active. If the node is not
   running, Greenlight automatically schedules and starts the node to
   handle the connection request.

 - Self-Hosted Nodes: For self-hosted nodes, the process is slightly
   different. The self-hosted node periodically reaches out to
   Greenlight's tunnel server. This tunnel server maintains a
   connection to the node, allowing the reverse proxy to forward
   incoming connections through the tunnel, effectively exposing the
   node to the internet. This approach eliminates the need for users
   to configure NAT or firewall settings, making it easier to maintain
   a self-hosted node.

In essence, the node domain provides a consistent and secure access
point for users' Lightning nodes, abstracting away the complexities of
node location and network configuration.

## Interpreting Errors

When using Server Name Indication (SNI) during the mutual TLS (mTLS)
handshake, the node domain server faces a limitation in its ability to
provide detailed error messages. This is because, during the mTLS
handshake, the server does not decrypt the data stream, limiting
itself to reading the unencrypted SNI fields in the handshake
packets. The SNI is used solely to identify the desired endpoint (the
specific node) before the encrypted stream is forwarded to the actual
destination. At this stage, the server has very limited means to
communicate back to the client, as the secure session has not yet been
fully established.

Due to these constraints, the server can only use the alert protocol
defined in RFC 5246, section 7.2, to report any issues encountered
during the scheduling process. The alert protocol is a part of the TLS
specification that allows the server to send predefined, minimal error
codes to the client, indicating that something has gone wrong. These
alerts are intentionally brief and do not provide extended error
details or context, which can make troubleshooting more difficult for
clients interacting with the node domain.

To mitigate this limitation, we utilize the custom range for alert
codes within the alert protocol to encode specific, common issues that
applications might encounter when connecting to the node domain. By
doing so, we provide a more granular indication of what might have
gone wrong during the connection attempt. While this approach still
operates within the constraints of the TLS handshake, it enables the
server to convey more specific errors than the standard TLS alerts,
assisting in diagnosing problems more effectively.

In summary, because the SNI is used before the data stream is
decrypted, the node domain server is restricted to using the TLS alert
protocol for error reporting. By leveraging custom alert codes, we can
offer more meaningful feedback within these constraints, helping
clients better understand and resolve connection issues.


### TLS Custom Alert Codes

 - 221 (`InternalError`): A catch-all server-side error. If we don't
   have more detailed information about what went wrong we return this
   code. It does not indicate an error on the user's side, rather an
   issue on the server that the Greenlight team needs to address.
 - 222 (`ReadSniError`): SNI (Server Name Indication, see [RFC 6066,
   Section3][rfc6066sec3]) is used to identify the desired node to
   talk to. If the node domain server is unable to parse the SNI
   header in the `ClientHelo` message this alert is returned. It
   usually means that you are trying to talk to the servers with a
   custom client. Please use the [`gl-client`][glclient] library.
 - 223 (`ConnectNodeError`): The node domain server has successfully
   located the node, but failed to connect to it. This may either be a
   stale scheduling in the hosted scenario, or a stale tunnel
   connection from the self-hosted node.
 - 224 (`ConnectSchedulerError`): The node domain server was unable to
   talk to the scheduler, and therefore could not locate, and/or
   schedule, the node.
 - 225 (`SchedulerError`): The node domain server contacted the
   scheduler correctly, but the scheduler was unable to locate or
   schedule the node.

Among these 221, 224 and 225 are considered server-side errors, and
should be handled by the Greenlight team. Please escalate them to the
team so we can identify the cause and address them. 222 and 223 are
considered to be user errors, as stale sessions on the hosted offering
are very unlikely, and therefore their appearance is mostly indicative
of a user error.

[rfc6066sec3]: https://datatracker.ietf.org/doc/html/rfc6066#section-3
[glclient]: https://github.com/Blockstream/greenlight
