# Webhooks

Webhooks allow your application to receive real-time HTTP notifications when events occur on your Greenlight nodes. Instead of polling for changes, Greenlight pushes event data to your endpoints as soon as events happen.

**Quick links:** [Event Types](#event-types) | [Signature Verification](#signature-verification) | [Managing Webhooks](#managing-webhooks) | [Best Practices](#best-practices)

## Webhook Types

Greenlight supports two types of webhooks that serve different use cases:

### Per-Node Webhooks

Per-node webhooks are registered via the Scheduler API and apply to a single node. Use these when you need fine-grained control over which endpoints receive events from specific nodes.

- Registered using `add_outgoing_webhook()` API
- Maximum **20 webhooks per node**
- Managed individually per node
- **Use case:** Monitoring specific nodes, per-customer webhook endpoints

### Per-Developer Webhooks

Per-developer webhooks are registered via the Greenlight Developer Console and automatically receive events from **all nodes** registered with your developer certificate.

- Registered in the Developer Console
- Applies to all nodes sharing your `referrer_pubkey`
- Single endpoint receives events from your entire fleet
- **Use case:** App-wide monitoring, centralized event processing for wallet apps

``` mermaid
graph TB
    subgraph "Your Application"
        EP1[Per-Node Endpoint<br/>node-a.example.com/webhook]
        EP2[Per-Node Endpoint<br/>node-b.example.com/webhook]
        EP3[App-Wide Endpoint<br/>app.example.com/webhooks]
    end

    subgraph "Per-Node Webhooks"
        NodeA[Node A] -->|events| EP1
        NodeB[Node B] -->|events| EP2
    end

    subgraph "Per-Developer Webhook"
        NodeA -.->|events| EP3
        NodeB -.->|events| EP3
        NodeC[Node C] -.->|events| EP3
    end

    DC[Developer Console] -->|registered| EP3
```

Both webhook types use the same payload format and signature scheme, so your endpoint code works identically regardless of how the webhook was registered.

---

## Event Types

### Payload Structure

All webhook payloads share a common structure:

```json
{
  "event_id": "7293847502938475029",
  "node_id": "02abc123def456...",
  "event_type": "invoice_payment",
  "timestamp": 1704067200,
  ...event-specific fields...
}
```

| Field | Type | Description |
|-------|------|-------------|
| `event_id` | string | Unique identifier (snowflake ID) for deduplication |
| `node_id` | string | Hex-encoded 33-byte compressed public key |
| `event_type` | string | Event type: `invoice_payment` or `node_stuck` |
| `timestamp` | integer | Unix timestamp (seconds) when the event occurred |

Additional fields are included depending on the event type.

---

### invoice_payment

Triggered when your node receives an incoming payment.

| Field | Type | Description |
|-------|------|-------------|
| `payment_hash` | string | Hex-encoded 32-byte payment hash |
| `preimage` | string | Hex-encoded 32-byte payment preimage (proof of payment) |
| `amount_msat` | integer | Amount received in millisatoshis |
| `bolt11` | string | The BOLT11 invoice that was paid |
| `label` | string | Invoice label (if set during creation) |

**Example payload:**

```json
{
  "event_id": "7293847502938475029",
  "node_id": "02a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
  "event_type": "invoice_payment",
  "timestamp": 1704067200,
  "payment_hash": "a1b2c3d4e5f67890abcdef1234567890abcdef1234567890abcdef1234567890",
  "preimage": "fedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321",
  "amount_msat": 100000,
  "bolt11": "lnbc1u1pjkx3xypp5...",
  "label": "order-12345"
}
```

**Common use cases:**

- Confirming payment receipt in e-commerce applications
- Triggering order fulfillment workflows
- Updating user balances in custody applications
- Sending payment confirmation notifications to users

---

### node_stuck

Triggered when a node falls behind the blockchain tip and cannot process new blocks.

| Field | Type | Description |
|-------|------|-------------|
| `blockheight` | integer | Node's current synced block height |
| `headheight` | integer | Current blockchain tip height |
| `lag` | integer | Number of blocks behind (`headheight - blockheight`) |

**Example payload:**

```json
{
  "event_id": "7293847502938475030",
  "node_id": "02a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2",
  "event_type": "node_stuck",
  "timestamp": 1704067300,
  "blockheight": 820000,
  "headheight": 820150,
  "lag": 150
}
```

**Common use cases:**

- Alerting operations teams to sync issues
- Temporarily pausing payment processing
- Triggering automated recovery procedures
- Monitoring node health dashboards

---

## Signature Verification

Every webhook request includes an HMAC-SHA256 signature in the `gl-signature` header. **Always verify this signature before processing the payload.**

### Signature Details

| Property | Value |
|----------|-------|
| Header | `gl-signature` |
| Algorithm | HMAC-SHA256 |
| Encoding | Base64 (standard alphabet) |
| Input | Raw request body bytes |
| Secret | Returned from `add_outgoing_webhook()` |

!!! warning "Verify Before Parsing"
    The signature is computed over the **raw request body bytes**. You must verify
    the signature before parsing JSON. Any whitespace changes or re-encoding
    will invalidate the signature.

### HTTP Headers

Greenlight sends these headers with every webhook request:

| Header | Description |
|--------|-------------|
| `Content-Type` | `application/json` |
| `gl-signature` | Base64-encoded HMAC-SHA256 signature |
| `X-Greenlight-Event` | Event type (e.g., `invoice_payment`) |

### Verification Examples

=== "Python"

    ```python
    import hmac
    import hashlib
    import base64
    from flask import Flask, request, abort

    app = Flask(__name__)
    WEBHOOK_SECRET = "your-webhook-secret"  # From add_outgoing_webhook()

    def verify_signature(payload: bytes, signature: str, secret: str) -> bool:
        """Verify the gl-signature header matches the payload."""
        expected = hmac.new(
            secret.encode("utf-8"),
            payload,
            digestmod=hashlib.sha256
        )
        computed = base64.b64encode(expected.digest()).decode("utf-8")
        return hmac.compare_digest(computed, signature)

    @app.route("/webhook", methods=["POST"])
    def handle_webhook():
        # Get raw body BEFORE parsing
        raw_body = request.get_data()
        signature = request.headers.get("gl-signature", "")

        if not verify_signature(raw_body, signature, WEBHOOK_SECRET):
            abort(401, "Invalid signature")

        # Safe to parse now
        event = request.get_json()
        
        # Handle event based on type
        if event["event_type"] == "invoice_payment":
            handle_payment(event)
        elif event["event_type"] == "node_stuck":
            handle_stuck_node(event)

        return "", 200
    ```

=== "Rust"

    ```rust
    use axum::{
        body::Bytes,
        http::{HeaderMap, StatusCode},
        routing::post,
        Router,
    };
    use base64::Engine;
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    const WEBHOOK_SECRET: &str = "your-webhook-secret";

    fn verify_signature(payload: &[u8], signature: &str, secret: &str) -> bool {
        let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
            return false;
        };
        mac.update(payload);
        let result = mac.finalize();

        let engine = base64::engine::general_purpose::STANDARD;
        let computed = engine.encode(result.into_bytes());

        // Constant-time comparison
        computed == signature
    }

    async fn handle_webhook(
        headers: HeaderMap,
        body: Bytes,
    ) -> StatusCode {
        let signature = headers
            .get("gl-signature")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !verify_signature(&body, signature, WEBHOOK_SECRET) {
            return StatusCode::UNAUTHORIZED;
        }

        // Safe to parse
        let event: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(e) => e,
            Err(_) => return StatusCode::BAD_REQUEST,
        };

        // Handle event...
        println!("Received event: {}", event["event_type"]);

        StatusCode::OK
    }
    ```

=== "Node.js"

    ```javascript
    const express = require('express');
    const crypto = require('crypto');

    const app = express();
    const WEBHOOK_SECRET = process.env.WEBHOOK_SECRET;

    function verifySignature(payload, signature, secret) {
        const hmac = crypto.createHmac('sha256', secret);
        hmac.update(payload);
        const computed = hmac.digest('base64');

        // Timing-safe comparison
        try {
            return crypto.timingSafeEqual(
                Buffer.from(computed),
                Buffer.from(signature)
            );
        } catch {
            return false;
        }
    }

    // Use raw body parser for signature verification
    app.post('/webhook', express.raw({ type: 'application/json' }), (req, res) => {
        const signature = req.headers['gl-signature'] || '';

        if (!verifySignature(req.body, signature, WEBHOOK_SECRET)) {
            return res.status(401).send('Invalid signature');
        }

        // Safe to parse
        const event = JSON.parse(req.body.toString());

        console.log(`Received ${event.event_type} for node ${event.node_id}`);

        // Handle event based on type
        switch (event.event_type) {
            case 'invoice_payment':
                handlePayment(event);
                break;
            case 'node_stuck':
                handleStuckNode(event);
                break;
        }

        res.sendStatus(200);
    });

    app.listen(3000);
    ```

---

## Delivery Guarantees

### Automatic Retries

Failed webhook deliveries are automatically retried with exponential backoff:

| Setting | Default Value |
|---------|---------------|
| Maximum attempts | 5 |
| Base delay | 60 seconds |
| Backoff multiplier | 2x |
| HTTP timeout | 30 seconds |

**Retry schedule:**

| Attempt | Delay After Failure |
|---------|---------------------|
| 1 | Immediate |
| 2 | 60 seconds |
| 3 | 2 minutes |
| 4 | 4 minutes |
| 5 | 8 minutes |

After 5 failed attempts, the event is dropped.

``` mermaid
sequenceDiagram
    participant S as Greenlight Service
    participant H as Webhook Dispatcher
    participant E as Your Endpoint

    S->>H: Event occurs
    H->>E: POST webhook
    
    alt Success (2xx)
        E-->>H: 200 OK
        Note over H: Delivery complete
    else Failure (5xx/timeout)
        E-->>H: 500 Error
        Note over H: Queue for retry
        H->>H: Wait 60s
        H->>E: Retry POST
        E-->>H: 200 OK
        Note over H: Delivery complete
    end
```

### What Triggers Retries

Deliveries are **retried** on:

- HTTP 5xx responses (server errors)
- Connection timeouts
- Network errors (connection refused, DNS failure)

Deliveries are **not retried** on:

- HTTP 2xx responses (success)
- HTTP 4xx responses (client errors - fix your endpoint)

!!! tip "Return 500 for Temporary Failures"
    If your endpoint encounters a temporary issue (database unavailable, rate limited),
    return HTTP 500 or 503 to trigger a retry. Return 4xx only for permanent failures
    like invalid signatures.

---

## Managing Webhooks

### Prerequisites

Before adding a webhook:

- A **public HTTPS endpoint** that can receive webhook events
- Access to your node's **device certificate**

### Adding a Webhook

Initialize a scheduler and register your webhook endpoint:

=== "Python"

    ```python
    from pathlib import Path
    from glclient import Credentials, Scheduler

    # Load credentials
    creds = Credentials.from_parts(
        Path("device.pem").read_bytes(),
        Path("device-key.pem").read_bytes(),
        Path("ca.pem").read_bytes(),
        Path("rune").read_bytes(),
    )

    node_id = bytes.fromhex("02a1b2c3...")

    scheduler = Scheduler(
        node_id=node_id,
        network="bitcoin",
        creds=creds
    )

    # Add webhook
    response = scheduler.add_outgoing_webhook("https://example.com/webhook")

    # Store the secret securely - it cannot be recovered!
    print(f"Webhook ID: {response.id}")
    print(f"Secret: {response.secret}")
    ```

=== "Rust"

    ```rust
    use gl_client::credentials::Builder;
    use gl_client::scheduler::Scheduler;
    use gl_client::bitcoin::Network;

    let credentials = Builder::as_device()
        .with_identity(device_cert, device_key)
        .build()
        .expect("Failed to build credentials");

    let scheduler = Scheduler::with_credentials(
        node_id,
        Network::Bitcoin,
        scheduler_uri,
        credentials
    ).await?;

    // Add webhook
    let response = scheduler
        .add_outgoing_webhook("https://example.com/webhook")
        .await?;

    // Store the secret securely - it cannot be recovered!
    println!("Webhook ID: {}", response.id);
    println!("Secret: {}", response.secret);
    ```

!!! warning "Secure Your Secret"
    The webhook secret is returned **only once** when you register the webhook.
    Store it securely in your secrets manager. If lost, you must delete the
    webhook and create a new one.

### Listing Webhooks

Retrieve all registered webhooks for a node:

=== "Python"

    ```python
    webhooks = scheduler.list_outgoing_webhooks()
    for webhook in webhooks.outgoing_webhooks:
        print(f"ID: {webhook.id}, URL: {webhook.uri}")
    ```

=== "Rust"

    ```rust
    let webhooks = scheduler.list_outgoing_webhooks().await?;
    for webhook in webhooks.outgoing_webhooks {
        println!("ID: {}, URL: {}", webhook.id, webhook.uri);
    }
    ```

!!! note
    Secrets are not included in the list response for security reasons.

### Deleting Webhooks

Remove webhooks by their IDs:

=== "Python"

    ```python
    scheduler.delete_outgoing_webhooks([1, 2, 3])
    ```

=== "Rust"

    ```rust
    scheduler.delete_webhooks(vec![1, 2, 3]).await?;
    ```

### Rotating Secrets

To rotate a webhook secret without downtime:

1. **Add a new webhook** with the same URL
2. **Update your endpoint** to accept both secrets temporarily
3. **Delete the old webhook** (or rotate its secret)
4. **Remove the old secret** from your endpoint

=== "Python"

    ```python
    # Option A: Add new webhook, delete old
    new_response = scheduler.add_outgoing_webhook("https://example.com/webhook")
    # Update your endpoint to use new_response.secret
    scheduler.delete_outgoing_webhooks([old_webhook_id])

    # Option B: Rotate existing webhook's secret
    response = scheduler.rotate_outgoing_webhook_secret(webhook_id)
    # response.secret contains the new secret
    ```

=== "Rust"

    ```rust
    // Rotate existing webhook's secret
    let response = scheduler
        .rotate_outgoing_webhook_secret(webhook_id)
        .await?;
    // response.secret contains the new secret
    ```

**Handling rotation in your endpoint:**

```python
# During rotation: accept multiple secrets
SECRETS = [
    os.environ["WEBHOOK_SECRET_OLD"],
    os.environ["WEBHOOK_SECRET_NEW"],
]

def verify_signature(payload: bytes, signature: str) -> bool:
    for secret in SECRETS:
        expected = hmac.new(secret.encode(), payload, hashlib.sha256)
        computed = base64.b64encode(expected.digest()).decode()
        if hmac.compare_digest(computed, signature):
            return True
    return False
```

---

## Best Practices

### Idempotency

Webhooks may be delivered more than once due to retries or network issues. Design your handlers to be idempotent using the `event_id` field.

```python
import redis

r = redis.Redis()
EVENT_TTL = 86400  # 24 hours

def handle_webhook(event):
    event_id = event["event_id"]

    # Check if already processed (atomic set-if-not-exists)
    if not r.set(f"webhook:{event_id}", "1", nx=True, ex=EVENT_TTL):
        return  # Already processed, skip

    # Process the event
    process_payment(event)
```

!!! tip "Event ID Retention"
    Keep event IDs for at least 24 hours to handle delayed retries.
    Redis with TTL or a database table with cleanup jobs works well.

### Response Time

Return a response quickly (within 30 seconds). Perform heavy processing asynchronously.

```python
from celery import Celery

celery = Celery('tasks', broker='redis://localhost')

@app.route("/webhook", methods=["POST"])
def handle_webhook():
    event = verify_and_parse(request)

    # Queue for background processing
    process_event.delay(event)

    # Return immediately
    return "", 200

@celery.task
def process_event(event):
    # Heavy processing happens here
    ...
```

### Error Handling

Use appropriate HTTP status codes:

| Response | When to Use |
|----------|-------------|
| `200 OK` | Event processed successfully |
| `202 Accepted` | Event received, processing async |
| `400 Bad Request` | Malformed payload (won't retry) |
| `401 Unauthorized` | Invalid signature (won't retry) |
| `500 Internal Server Error` | Temporary failure (will retry) |
| `503 Service Unavailable` | Overloaded (will retry) |

### Security

- **HTTPS required** - Webhook endpoints must use HTTPS with valid certificates
- **Verify signatures** - Always verify the `gl-signature` header before processing
- **Validate early** - Verify signatures before parsing JSON
- **Use constant-time comparison** - Prevents timing attacks on signature verification
- **Store secrets securely** - Use environment variables or a secrets manager

### Testing

For local development, use a tunneling service:

```bash
# Using ngrok
ngrok http 8000

# Register the ngrok URL as your webhook endpoint
# https://abc123.ngrok.io/webhook
```

Test signature verification independently:

```python
def test_signature_verification():
    secret = "test-secret"
    payload = b'{"event_id":"123","event_type":"invoice_payment"}'

    # Compute expected signature
    expected = base64.b64encode(
        hmac.new(secret.encode(), payload, hashlib.sha256).digest()
    ).decode()

    assert verify_signature(payload, expected, secret)
    assert not verify_signature(payload, "wrong", secret)
    assert not verify_signature(b"tampered", expected, secret)
```
