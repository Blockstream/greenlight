
# Getting started

## Installation

We are currently distributing the package via a custom scope
`@greenlight` on a private repository. Without additional configration
`npm` would go look for our packages in the public repository, so
you'll have to tell it where to find the private repository first:

```bash
npm config set @greenlight:registry=https://us-west2-npm.pkg.dev/c-lightning/test-npm/
```

Alternatively you can also create an `.npmrc` file in your project's
root with the following content to automatically apply this mapping
for all devs on this project:

```text
@greenlight:registry = "https://us-west2-npm.pkg.dev/c-lightning/test-npm/"
```

This tells `npm` where to find the repository for packages in the
`@greenlight` scope. Afterwards `npm` calls involving scoped packages will use the private repo.

Installing the actual dependency is done via:

```bash
npm install @greenlight/gl-client-js
```

And it should automatically pull down a precompiled binary image of
the library, skipping the lengthy compilation process. If this fails
we likely just haven't compiled the library for your architecture and
platform, so please let us know by opening an issue. If you have all
the dependencies (see [`libhsmd`](../gl-client-py) for a list) you
might be able to compile the binary extension from its source,
automatically when `npm install`ing or `npm update`ing in your project
(though that may take a while).

## Updating

After following the configuration steps above, pointing the
`@greenlight` scope to the private repository, `npm update` should
just work:

```bash
npm update
```

## Examples

The following examples show how the API can be used to talk to greenlight:

```javascript
const glclient = require('@greenlight/gl-client-js');
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
const glclient = require('@greenlight/gl-client-js');
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
