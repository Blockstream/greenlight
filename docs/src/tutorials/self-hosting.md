# Self-Hosting Your Node

It is our goal to help you get started with the Lightning
Network. However once you learned all that is required to run your own
node, it is time to level up and approach the next challenge: managing
your own node.

For this purpose Greenlight allows users to export their nodes and
restore it on their own infrastructure. This process can take a couple
of minutes to complete, but it is faster and cheaper than migrating by
shutting down the old node on GL, transferring all your funds, and
having to bootstrap a new node from scratch.

Under the hood we will:

 1. Mark the node as exported, so it doesn't start on GL going forward.
 2. Initiate a DB backup of the node's wallet database.
 3. Encrypt the DB backup so that it can be decrypted only by users
    that have access to the seed phrase.
 4. Store the encrypted backup on a file store, from where it can be
    downloaded, decrypted and restored locally by the user.

Notice that step 1, marking the node as not-schedulable, is important
for the security of the funds: if the node on GL were allowed to make
progress it'd invalidate the backup, which could lead to channel
closures.

## Setup Your Infrastructure

Before initiating the export, we will first need to prepare the new
home where the node will be running going forward. The node running on
Greenlight is a slightly modified Core Lightning node, with some of
custom subdaemons and plugins. You can chose to replicate the same
node by also running the custom subdaemons and plugins, or you can run
a vanilla Core Lightning node. Custom components include the
following:

 - `signerproxy` subdaemon: exposes the signer interface to the
   signer, enabling the remote signer setup.
 - `gl-plugin` plugin: exposes the plugin interface, including mTLS
   verification, and acts as a tailable signer request stream.
   
If you'd like to continue using the remote signer you should use both
`gl-plugin` and `signerproxy` in your own setup as well. If not you
will have to create the `hsm_secret` file in the CLN config directory
so that the stock CLN signer can find it and use it to run a
local-only signer.

If you have any clients configured you'll likely want to run the
`gl-plugin` rather than the stock `grpc-plugin`, as the former
implements a superset of the latter. In addition clients will still be
able to reach the node through the node URL through the [GL reverse
proxy][node-rpc-proxy]. In the next sections we provide step-by-step
instructions to restore the node's database, and then setup the
different variants. You will have to decide which one best suites your
needs :wink:

### Restoring the database

<!-- TODO -->

### Remote Signer Setup

<!-- TODO -->

### Local Signer Setups

<!-- TODO -->

### Minimal Setup

The minimal setup to host an exported node consists of a
[PostgreSQL][postgres] database, and a CLN installation matching or
newer than the version that was running on Greenlight. Assuming the
database is running locally with the credentials `pguser` and `pgpass` you'
you'll have to run CLN node like this:

```sh
lightningd --wallet=postgres://pguser:pgpass@localhost:5432/dbname
```

## Initiate Export

The export can be triggered via the `Scheduler.export_node` method on
the [Scheduler's grpc interface][schedpb]. This method call may take a
couple of minutes depending on the age of the node, the number of
channels opened and closed, as well as the number of payments sent and
received. Upon successful export the call will return a URL from a
file host where the encrypted backup can be downloaded.


!!! warning

	Once the state of the node has been switched you will no longer be able to
	schedule the node on Greenlight's infrastructure. We do not allow 
	re-activating the node because of the risk of a replica running somewhere 
	else, which could cause loss of funds!

Do not worry if the connection is lost during this process, as the
method is idempotent and will complete in the background if the
connection is lost. Calling the method multiple times will results in
the same encrypted backup URL

[postgres]: https://www.postgresql.org/
[schedpb]: https://github.com/Blockstream/greenlight/blob/main/libs/proto/scheduler.proto
[node-rpc-proxy]: ../reference/node-rpc-proxy.md
