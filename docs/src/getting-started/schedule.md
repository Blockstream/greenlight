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
	use hex;
	let node_id = hex::decode("02058e8b6c2ad363ec59aa136429256d745164c2bdc87f98f0a68690ec2c5c9b0b")?;
	let network = "testnet";
	
	let scheduler = gl_client::scheduler::Scheduler(node_id, network)?;
	let node = scheduler.schedule()?;
	```

=== "Python"
	```python
	from glclient import TlsConfig, Scheduler, 
	cert, key = b'...', b'...'
	node_id = bytes.fromhex("02058e8b6c2ad363ec59aa136429256d745164c2bdc87f98f0a68690ec2c5c9b0b")
	network = "testnet"
	tls = TlsConfig.with_identity(cert, key)
	
	scheduler = Scheduler(node_id, network, tls)
	node = scheduler.node()
	```

=== "Javascript"
	<!-- TODO -->

Once we have an instance of the `Node` we can start interacting with it via the GRPC interface:

=== "Rust"
	<!-- TODO -->
=== "Python"
	<!-- TODO -->
=== "Javascript"
	<!-- TODO -->

