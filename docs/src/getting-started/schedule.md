# Starting a node

Now that the node has been registered on the Greenlight server, we can
schedule it. Scheduling will tell the scheduler that we want to
interact with the node, and need its GRPC URI, so we can talk to
it. The scheduler will look up the node, check if it is currently
running, and if not it'll start the node. It will then return the URL
you can use to connect to the node directly.

!!! important
	Currently nodes will get a new address whenever they are started,
	so don't cache the URL for longer periods of time. We spin nodes
	down if there is no client talking to it, and slots are reused for
	other nodes. Attempting to talk to a node that isn't yours will
	fail to establish a connection.

	The Greenlight team is working on an improvement that will
	assign a unique address to each node, ensuring that you always
	know how to reach the node, and allowing you to skip talking with
	the scheduler altogether.
	
First of all we build an instance of the scheduler service stub, which
will allow us to call methods on the service. We then schedule the
node, which returns a stub representing the node running on the
Greenlight infrastructure:

=== "Rust"
	```rust
--8<-- "getting_started.rs:start_node"
	```

=== "Python"
	```python
--8<-- "main.py:start_node"
	```

Once we have an instance of the `Node` we can start interacting with it via the GRPC interface:

=== "Rust"
    ```rust
--8<-- "getting_started.rs:list_peers"
	```
=== "Python"
	```python
--8<-- "main.py:list_peers"
	```
	
The above snippet will read the metadata and list the peers from the
node. Both of these are read-only operations, that do not require a
signer to sign off. What happens if we issue a command that requires a
signer to sign off? Let's try to connect to create an
invoice. Invoices are signed using the node key, and the signer is the
only component with access to your key.

=== "Rust"
	```rust
--8<-- "getting_started.rs:create_invoice"
	```

=== "Python"
	```python
--8<-- "main.py:create_invoice"
	```
	
You'll notice that these calls hang indefinitely. This is because the
signer is not running and not attached to the node, and without its
signature we can't create the invoice. This isn't just the case for
the `invoice` call either, all calls that somehow use the Node ID, or
move funds, will require the signer's sign-off. You can think of a
node without a signer being connected as a read-only node, and as soon
as you attach the signer, the node becomes fully functional. So how do
we attach the signer? Simple: load the secret from where you stored it
in the last chapter, instantiate the signer with it and then start it.

=== "Rust"
	```rust
--8<-- "getting_started.rs:start_signer"
	```
	
	Notice that `signer.run_forever()` returns a `Future` which you can spawn a
	new task with. That is also the reason why a separate shutdown signal is
	provided.
	
=== "Python"
	```python
--8<-- "main.py:start_signer"
	```

If you kept the stuck commands above running, you should notice that
they now return a result. As mentioned before many RPC calls will need
the signer to be attached to the node, so it's best to just start it
early, and keep it running in the background whenever possible. The
signer will not schedule the node by itself, instead waiting on the
scheduler, so it doesn't consume much resources, but still be
available when it is needed.
