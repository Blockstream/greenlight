# Greenlight SDK - Node.js Bindings

Node.js bindings for Blockstream's Greenlight SDK using [napi-rs](https://napi.rs/).

## Prerequisites

- Node.js >= 16
- Protocol Buffers compiler (`protoc`)

## Installation

### From npm

```bash
npm install @blockstream/gl-sdk
```

### Building from source

1. Clone the Greenlight repository:

```bash
git clone https://github.com/Blockstream/greenlight.git
cd greenlight
```

2. Navigate to the napi bindings directory:

```bash
cd libs/gl-sdk-napi
```

3. Install dependencies:

```bash
npm install
```

4. Build the package:

```bash
npm run build
```

This will compile the Rust code and generate the Node.js native module.

## Project Structure

```
gl-sdk-napi/
├── Cargo.toml          # Rust dependencies and configuration
├── package.json        # Node.js package configuration
├── src/
│   └── lib.rs          # Rust implementation of Node.js bindings
├── example.ts          # TypeScript usage examples
└── tests/              # Test file/s
```

## Usage Example

```typescript
import { Scheduler, Signer, Node, Credentials } from '@blockstream/gl-sdk';
import { randomBytes } from 'crypto';

async function quickStart() {
  // 1. Register a new node
  const scheduler = new Scheduler('regtest');
  const seed = randomBytes(32);
  
  const regResult = await scheduler.register(seed, 'INVITE_CODE');
  const { device_cert, device_key } = JSON.parse(regResult);
  
  // 2. Create credentials
  const credentials = new Credentials(
    Buffer.from(device_cert),
    Buffer.from(device_key)
  );
  
  // 3. Start the signer
  const signer = new Signer(seed, 'regtest', credentials);
  await signer.start();
  
  // 4. Connect to the node
  const nodeId = await signer.nodeId();
  const scheduleInfo = await scheduler.schedule(nodeId);
  const { grpc_uri } = JSON.parse(scheduleInfo);
  
  const node = await Node.connect(credentials, grpc_uri);
  
  const address = await node.onchainReceive();
  console.log('New address:', address.bech32);
  
  // 6. Cleanup
  await signer.stop();
}

quickStart().catch(console.error);
```

## Development

### Running Tests

```bash
npm test
```

### Building for Production

```bash
npm run build
```

### Cross-compilation

Build for multiple platforms:

```bash
npm run build -- --target x86_64-unknown-linux-gnu
npm run build -- --target aarch64-apple-darwin
npm run build -- --target x86_64-pc-windows-msvc
```

## Important Notes

1. **Async/Await**: All network operations are asynchronous. Always use await or handle returned promises properly to avoid unhandled rejections or unexpected behavior.

2. **Napi Macros**: We do not combine N-API (napi) macros with UniFFI macros in the same crate. They are incompatible, and mixing them within a single crate would cause conflicts and break the UniFFI-generated mobile bindings.

## Resources

- [Greenlight Documentation](https://blockstream.github.io/greenlight/)
- [napi-rs Documentation](https://napi.rs/)
- [Prebuilt Binary Support Matrix](https://github.com/napi-rs/napi-rs?tab=readme-ov-file#msrv)
- [Features Table](https://github.com/napi-rs/napi-rs?tab=readme-ov-file#features-table)