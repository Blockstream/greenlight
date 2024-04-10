## Webhooks
Webhooks are URLs that receive HTTP requests containing event-related data. The application sending the event could be part of the hosting application or a different application entirely.

With greenlight, developers can use webhooks to subscribe to events related to a given node. Up to 20 webhooks can be added per node and duplicate urls are permitted to help facilitate secret rotations.

## Events

Events are sent as HTTP POST requests with json payloads containing the details of the event. Events are structured in the following format:

```
{
  "version": <version>,
  "node_id": <node_id>,
  "event_type": <event_type>
}
```

## Adding a webhook to a greenlight node

### Prerequisites
- A public tls-secured endpoint 
- Access to a greenlight node's device certificate

To add a webhook for a greenlight node we first need to initialize a scheduler using the node id and device certificate (returned in the node registration response).

=== "Rust"
	```rust
	let device_cert = include_bytes!("path-to-device-cert");
	let device_key = include_bytes!("path-to-device-key");

	let credentials = Builder::as_device()
		.with_identity(device_cert, device_key)
		.build()
		.expect("Failed to build Device credentials");

	let node_id = hex::decode("hex-node-id").unwrap();

	let scheduler = Scheduler::with_credentials(
		node_id,
		gl_client::bitcoin::Network::Bitcoin,
		utils::scheduler_uri(),
		credentials
	)
	.await
	.unwrap();
	```

=== "Python"
	```python
	from pathlib import Path
	from glclient import Credentials, TlsConfig

	certpath = Path("device.pem")
	keypath = Path("device-key.pem")
	capath = Path("ca.pem")
	runepath = Path("rune")

	creds = Credentials.from_parts(
		certpath.open(mode="rb").read(),
		keypath.open(mode="rb").read(),
		capath.open(mode="rb").read(),
		runepath.open(mode="rb").read(),
	)
	
	node_id = bytes.fromhex("hex-node-id")
	
	scheduler = scheduler = Scheduler(
		node_id=node_id,
		network="bitcoin",
		creds=creds
	)
	```

Once we're able to initialize the scheduler, we simply need to call `add_outgoing_webhook` with a well-formed url to finish adding the webhook. 

!!! warning "Don't forget to secure your webhook secrets"

	Notice that we call `save_secret_to_db` to save the secret needed to validate webhook requests for this node. This secret cannot be recovered if lost and must be securely stored immediately after the addition of the webhook.

=== "Rust"
	```rust
	use gl_client::scheduler::Scheduler;
	use gl_client::bitcoin::Network;

	let webhook_uri = "https://example.com";
	let add_webhook_response = scheduler.add_outgoing_webhook(webhook_uri).await.unwrap();

	save_secret_to_db(signer.node_id(), &add_webhook_response.secret);
	```

=== "Python"
	```python
	from glclient import Scheduler
	
	scheduler = Scheduler(
		node_id=signer.node_id(),
		network="bitcoin",
		tls=tls,
	)
	
	webhook_uri = "https://example.com"
	scheduler.add_outgoing_webhook(webhook_uri)

	save_secret_to_db(signer.node_id(), add_webhook_response.secret);
	```

## Verifying webhook payloads

Webhook payloads can be verified using the secret returned from `add_outgoing_webhook`. The secret can not be shared across nodes and is only valid for the node it was returned for. The secret serves as the key needed to validate the payload in an hmac-sha256 hash. If the payload is valid, the resulting hash should be equal to the base58-encoded value contained within the 'gl-signature' header of the request sent to the webhook.

=== "Rust"
	```rust
	use base64::Engine;
	use hmac::{Hmac, Mac};
	use sha2::Sha256;

	fn verify_signature(secret: &String, gl_signature: &String) -> Result<bool> {
		let mut hmac = match Hmac::<Sha256>::new_from_slice(secret.as_bytes()) {
				Ok(m) => m,
				Err(e) => Err(anyhow!("{:?}", e))
		};

		hmac.update(&message.as_bytes());
		let hmac_output_bytes = hmac.finalize().into_bytes();

		let engine = base64::engine::general_purpose::STANDARD;
		match engine.encode(&hmac_output_bytes) {
			Ok(generated_signature) => Ok(generated_signature == gl_signature),
			Err(e) => Err(anyhow!("{:?}", e))
		}
	}
	```

=== "Python"
	```python
	import hmac, hashlib, base64

	def verify_signature(secret: str, body, sig) -> bool:
		payload_hmac = hmac.HMAC(
				bytes(secret, "UTF-8"), body, digestmod=hashlib.sha256
		)
		base64_encoded_payload_hmac = base64.b64encode(
				payload_hmac.digest()
		)
		return base64_encoded_payload_hmac.decode() == sig
	```

## Listing webhooks 

Registered webhooks are uniquely identified for each node using an identifier called the webhook id. Calling `list_outgoing_webhooks` with a scheduler using the targeted node's device certificate will list all webhooks urls along with their uniquely generated webhook ids. This information is needed to delete a node's webhooks.

=== "Rust"
	```rust
	let outgoing_webhooks = scheduler.list_outgoing_webhooks().await.unwrap();
	```

=== "Python"
	```python
	outgoing_webhooks = scheduler.list_outgoing_webhooks();
	```

## Deleting webhooks

Webhooks can be deleted by passing a webhook id to the scheduler's `delete_outgoing_webhook` or a list of webhook ids to `delete_outgoing_webhooks`.

=== "Rust"
	```rust
	let outgoing_webhooks = scheduler.delete_outgoing_webhooks(vec![1,2,3]).await.unwrap();
	```

=== "Python"
	```python
	scheduler.delete_outgoing_webhooks([1,2,3]);
	```

## Rotating webhook secrets

Rotating a webhook's secret invalidates the secret for a webhook id and generates a new secret that is then returned in the response. Once the rotate method is invoked, the existing secret is immediately invalidated. The recommended process for updating a webhook's secret is to add another webhook with the same url, rotate the old one, and delete the new one if a static webhook id is needed. If not, the old webhook id can be deleted instead of the new one in the last step.

!!! note
	The payload must be hashed and verified *before* it's parsed as JSON to avoid any additional bytes from
	being added before signature validation. It's very important that the payload is not altered in anyway
	before its bytes are used as input to the HMAC.

=== "Rust"
	```rust
	let secret_response = scheduler.rotate_outgoing_webhook_secret(1).await.unwrap();
	```

=== "Python"
	```python
	secret_response = scheduler.rotate_outgoing_webhook_secret(1);
	```