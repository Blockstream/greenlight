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
	
	let tls = TlsConfig::new().unwrap().identity(device_cert, device_key);

	let scheduler = gl_client::scheduler::Scheduler(node_id, network)?;
	let node: gl_client::node::ClnClient = scheduler.schedule(tls).await?;
	```

=== "Python"
	```python
	from glclient import TlsConfig, Scheduler, 
	cert, key = b'...', b'...'
	node_id = bytes.fromhex("02058e8b6c2ad363ec59aa136429256d745164c2bdc87f98f0a68690ec2c5c9b0b")
	network = "testnet"
	tls = TlsConfig().identity(cert, key)
	
	scheduler = Scheduler(node_id, network, tls)
	node = scheduler.node()
	```

Once we have an instance of the `Node` we can start interacting with it via the GRPC interface:

=== "Rust"
    ```rust
    use gl_client::pb::cln;
	let info = node.get_info(cln::GetinfoRequest::default()).await?;
	let peers = node.list_peers(gl_client::pb::cln::ListpeersRequest::default()).await?;
	```
=== "Python"
	```python
	info = node.get_info()
	peers = node.list_peers()
	```
	
The above snippet will read the metadata and list the peers from the
node. Both of these are read-only operations, that do not require a
signer to sign off. What happens if we issue a command that requires a
signer to sign off? Let's try to connect to create an
invoice. Invoices are signed using the node key, and the signer is the
only component with access to your key.

=== "Rust"
	```rust
    node.invoice(cln::InvoiceRequest {
	    label: "label".to_string(),
		description: "description".to_string(),
		..Default::default(),
	}).await?;
	```

=== "Python"
	```python
	from glclient import clnpb
	node.invoice(
	    amount_msat=clnpb.AmountOrAny(any=True),
		label="label",
		description="description",
	)
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
	let seed = ... // Load from wherever you stored it
	let (cert, key) = ... // Load the cert and key you got from the `register` call
	
	// The signer task will run until we send a shutdown signal on this channel
	let (tx, mut rx) = tokio::sync::mpsc::channel(1);
	
	let tls = TlsConfig().identity(cert, key);
	signer = Signer(seed, Network::Bitcoin, tls);
	signer.run_forever(rx).await?;
	```
	
	Notice that `signer.run_forever()` returns a `Future` which you can spawn a
	new task with. That is also the reason why a separate shutdown signal is
	provided.
	
=== "Python"
	```python
	seed = ... # Load from wherever you stored it
	cert, key = ... // Load the cert and key you got from the `register` call
	
	tls = TlsConfig().identity(cert, key)
	signer = Signer::new(secret, Network::Bitcoin, tls)
	signer.run_in_thread()
	```

If you kept the stuck commands above running, you should notice that
they now return a result. As mentioned before many RPC calls will need
the signer to be attached to the node, so it's best to just start it
early, and keep it running in the background whenever possible. The
signer will not schedule the node by itself, instead waiting on the
scheduler, so it doesn't consume much resources, but still be
available when it is needed.
