## Webhooks

Webhooks are URLs that receive HTTP requests containing event-related data. They allow applications to notify external systems of important events in real-time. The event source can be part of the same hosting application or an entirely different system.

With Greenlight, you can use webhooks to subscribe to events related to a given node. Each node supports up to **20 webhooks**, and duplicate URLs are allowed to facilitate secret rotations.

## Events

Events are sent as HTTP `POST` requests with JSON payloads containing event details. The payload structure is as follows:

```json
{
  "version": <version>,
  "node_id": <node_id>,
  "event_type": <event_type>
}
```

## Adding a Webhook to a Greenlight Node

### Prerequisites

Before adding a webhook, ensure the following:

- A **public TLS-secured endpoint** that can receive webhook events.
- Access to a Greenlight node's **device certificate**.

### Step 1: Initialize a Scheduler

To add a webhook, first initialize a scheduler using the node ID and device certificate (obtained during node registration).

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
from glclient import Credentials, Scheduler

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

scheduler = Scheduler(
    node_id=node_id,
    network="bitcoin",
    creds=creds
)
```

### Step 2: Add the Webhook

Once the scheduler is initialized, add a webhook using `add_outgoing_webhook`. Ensure that your webhook URL is correctly formatted.

!!! warning "Secure Your Webhook Secret"
    The `add_outgoing_webhook` method returns a **secret** used for webhook request validation. 
    Store this securely as it **cannot be recovered** if lost.

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
scheduler.add_outgoing_webhook("https://example.com")

save_secret_to_db(signer.node_id(), add_webhook_response.secret)
```

## Verifying Webhook Payloads

Webhook payloads are verified using the **secret** returned from `add_outgoing_webhook`. This secret is unique per node and is used to validate payloads via **HMAC-SHA256 hashing**. The generated hash should match the **base64-encoded** value in the `gl-signature` header.

=== "Rust"
```rust
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn verify_signature(secret: &String, gl_signature: &String) -> Result<bool> {
    let mut hmac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("Failed to create HMAC");
    hmac.update(&message.as_bytes());
    let hmac_output_bytes = hmac.finalize().into_bytes();
    
    let engine = base64::engine::general_purpose::STANDARD;
    Ok(engine.encode(&hmac_output_bytes) == *gl_signature)
}
```

=== "Python"
```python
import hmac, hashlib, base64

def verify_signature(secret: str, body, sig) -> bool:
    payload_hmac = hmac.new(
        bytes(secret, "UTF-8"), body, digestmod=hashlib.sha256
    )
    return base64.b64encode(payload_hmac.digest()).decode() == sig
```

## Managing Webhooks

### Listing Webhooks

To retrieve registered webhooks for a node, use `list_outgoing_webhooks`.

=== "Rust"
```rust
let outgoing_webhooks = scheduler.list_outgoing_webhooks().await.unwrap();
```

=== "Python"
```python
outgoing_webhooks = scheduler.list_outgoing_webhooks()
```

### Deleting Webhooks

To delete webhooks, pass the webhook ID(s) to `delete_outgoing_webhook`.

=== "Rust"
```rust
scheduler.delete_outgoing_webhooks(vec![1,2,3]).await.unwrap();
```

=== "Python"
```python
scheduler.delete_outgoing_webhooks([1,2,3])
```

## Rotating Webhook Secrets

Rotating a webhook secret **invalidates** the old secret and generates a new one. To ensure a smooth transition:

1. Add a **new webhook** with the same URL.
2. Rotate the **old webhook secret**.
3. Delete either the new or old webhook, depending on whether you need a **static webhook ID**.

!!! note
    Ensure payload verification occurs **before** JSON parsing. Any modification to the payload before hashing will invalidate the signature.

=== "Rust"
```rust
let secret_response = scheduler.rotate_outgoing_webhook_secret(1).await.unwrap();
```

=== "Python"
```python
secret_response = scheduler.rotate_outgoing_webhook_secret(1)
```
