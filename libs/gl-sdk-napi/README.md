# Greenlight SDK - Node.js Bindings

Node.js bindings for Blockstream's Greenlight SDK using [napi-rs](https://napi.rs/).

## Prerequisites

- Node.js >= 16
- Protocol Buffers compiler (`protoc`)

## Installation

### From npm

```bash
npm install @greenlightcln/glsdk
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

## Supported Platforms

Prebuilt binaries are available for the following platforms:

| OS      | Architecture | Target                      |
|---------|--------------|-----------------------------|
| Linux   | x64          | x86_64-unknown-linux-gnu    |
| Linux   | arm64        | aarch64-unknown-linux-gnu   |
| macOS   | x64          | x86_64-apple-darwin         |
| macOS   | arm64 (M1/M2)| aarch64-apple-darwin        |
| Windows | x64          | x86_64-pc-windows-msvc      |

### For Unsupported Platforms:

1. Follow the instructions in the [Building from source](#building-from-source) section above.

2. Then install the package from your local directory:

```bash
cd /path/to/your/project
npm install /path/to/greenlight/libs/gl-sdk-napi
```

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

**Async/Await**: All network operations are asynchronous. Always use await or handle returned promises properly to avoid unhandled rejections or unexpected behavior.


```typescript
import { Scheduler, Signer, Node, Credentials, OnchainReceiveResponse } from '@greenlightcln/glsdk';

type Network = 'bitcoin' | 'regtest';

class GreenlightApp {
  private credentials: Credentials | null = null;
  private scheduler: Scheduler;
  private signer: Signer;
  private node: Node | null = null;

  constructor(phrase: string, network: Network) {
    this.scheduler = new Scheduler(network);
    this.signer = new Signer(phrase);
    const nodeId = this.signer.nodeId();
    console.log(`✓ Signer created. Node ID: ${nodeId.toString('hex')}`);
  }

  async registerOrRecover(inviteCode?: string): Promise<void> {
    try {
      console.log('Attempting to register node...');
      this.credentials = await this.scheduler.register(this.signer, inviteCode || '');
      console.log('✓ Node registered successfully');
    } catch (e: any) {
      const match = e.message.match(/message: "([^"]+)"/);
      console.error(`✗ Registration failed: ${match ? match[1] : e.message}`);
      console.log('Attempting recovery...');
      try {
        this.credentials = await this.scheduler.recover(this.signer);
        console.log('✓ Node recovered successfully');
      } catch (recoverError) {
        console.error('✗ Recovery failed:', recoverError);
        throw recoverError;
      }
    }
  }

  createNode(): Node {
    if (!this.credentials) { throw new Error('Must register/recover before creating node'); }
    console.log('Creating node instance...');
    this.node = new Node(this.credentials);
    console.log('✓ Node created');
    return this.node;
  }

  async getOnchainAddress(): Promise<OnchainReceiveResponse> {
    if (!this.node) { this.createNode(); }
    console.log('Generating on-chain address...');
    return await this.node!.onchainReceive();
  }

  async cleanup(): Promise<void> {
    if (this.node) {
      await this.node.stop();
      console.log('✓ Node stopped');
    }
  }
}

async function quickStart(): Promise<void> {
  const phrase = 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about';
  const network: Network = 'regtest';
  console.log('=== Greenlight SDK Demo ===');
  const app = new GreenlightApp(phrase, network);
  try {
    await app.registerOrRecover();
    app.createNode();
    const address = await app.getOnchainAddress();
    console.log(`✓ Bech32 Address: ${address.bech32}`);
    console.log(`✓ P2TR Address: ${address.p2Tr}`);
  } catch (e) {
    console.error('✗ Error:', e);
  } finally {
    await app.cleanup();
  }
}

quickStart();
```

## Development

### Running Tests

```bash
npm test
```

### Local npm Publishing
This workflow only builds for local platform. For multi-platform builds, use the GitHub Actions workflow which cross-compiles for all supported targets.

```bash
# Clean previous builds
npm run clean

# Build the native binary for your platform
npm run build

# Preview what will be included in the package (Tarball Contents)
npm pack --dry-run

# Bump version (patch: 0.1.4 → 0.1.5, minor: 0.1.4 → 0.2.0, major: 0.1.4 → 1.0.0)
npm version patch/minor/major

# Publish to npm registry (requires authentication)
npm publish --access public
```

## Resources

- [Greenlight Documentation](https://blockstream.github.io/greenlight/)
- [napi-rs Documentation](https://napi.rs/)
- [Prebuilt Binary Support Matrix](https://github.com/napi-rs/napi-rs?tab=readme-ov-file#msrv)
- [Features Table](https://github.com/napi-rs/napi-rs?tab=readme-ov-file#features-table)