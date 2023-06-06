
# Getting started

## Installation

Installing the dependency is done via `npm`:

```bash
npm install gl-client
```

And it should automatically pull down a precompiled binary image of
the library, skipping the lengthy compilation process. If this fails
we likely just haven't compiled the library for your architecture and
platform, so please let us know by opening an issue. You might be able
to compile the binary extension from its source, automatically when
`npm install`ing or `npm update`ing in your project (though that may
take a while).

## Updating

`npm update` should just work:

```bash
npm update
```

## Examples

The following examples show how the API can be used to talk to greenlight:

```javascript
const glclient = require('gl-client');
const buffer = require("buffer");

// The scheduler accepts connections with identity /users/nobody for `register` and `recover`
let tls = new glclient.TlsConfig();

// We assume you're storing the secret somewhere safe! Don't store it in the code like this
let signer = new glclient.Signer(
  buffer.Buffer.from("00000000000000000000000000000000", "utf8"),
    "regtest",
    tls
);

let node_id = signer.node_id();
let sched = new glclient.Scheduler(node_id, "regtest");

let response = sched.register(signer);
console.log(response);
```

The following allows you to schedule the node on our infrastructure:

```js
const glclient = require('gl-client');
const buffer = require("buffer");

// Notice this time we have to load an identity that corresponds to the node, this is
// because we want to schedule the node and then talk to it.
let tls = new glclient.TlsConfig();
let user_tls = tls.load_file("device.crt", "device-key.pem");

let signer = new glclient.Signer(
  buffer.Buffer.from("00000000000000000000000000000000", "utf8"),
  "regtest",
  user_tls
);

let node_id = signer.node_id();
console.log("Node ID", node_id);
let sched = new glclient.Scheduler(node_id, "regtest", user_tls);

let node = sched.schedule();

// Now you can use `node` as if it were any other c-lightning node:
console.log(node.get_info())

node.stop();
```
