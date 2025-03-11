# gl-cli

`gl-cli` is a command-line interface for running a __Greenlight signer__ and 
operating a __Greenlight node__. It is built on top of the `gl-client` library.
The crate is called `gl-cli` to be consistent with the naming scheme that
__Greenlight__ uses but the binary it produces is `glcli` for convenience
reasons.

## Features

`glcli` is not yet feature-complete but already provides a basic set of
commands necessary for everyday node operations. Planned future enhancements
include additional commands and broader integration.

* __Scheduler__: Interact with Greenlight's scheduler to provision and start nodes.
* __Signer__: Run and interact with a local signer.
* __Node__: Operate and control a lightning node hosted on Greenlight.

## Installation

You can install `glcli` from __crates.io__
```bash
cargo install gl-cli
```

### Prerequisites

Ensure __Rust__ is installed on your system. If it is not installed, you can set
it up using [Rustup]("https://rustup.rs/"):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build and Install Locally

Navigate to the `glcli` directory and use __Cargo__ to install `glcli` into
your `CARGO_HOME` (which defaults to `$HOME/.cargo/`):
```bash
cargo install --path=.
```
Ensure that `CARGO_HOME/bin` is in your `PATH`.

## Usage

After installation, run:
```bash
glcli --help
```
This will display an overview of the available commands.

By default, `glcli` stores application data in the system's default data 
directory:
* __Linux__: `$XDG_DATA_HOME` or `$HOME/.local/share`
* __macOS__: `$HOME`/Library/Application Support

To specify a custom data directory, use the `--data-dir` or `-d` option. For
example:
```bash
glcli -d "${HOME}/.greenlight_node_1" node getinfo
```

### Register a Node

Before you can operate a Greenlight node you have to register one. Currently, 
you need an _invite code_ or a
[developer certificate]("https://blockstream.github.io/greenlight/getting-started/certs/")
to register a new node. `glcli` currently supports registration via an
_invite code_ using the `--invite-code` option:
```bash
glcli scheduler register --invite-code=<YOUR_INVITE_CODE>
```

### Run a Local Signer

To operate your node, you need to attach a local signer to your Greenlight node. 
The signer is responsible for handling cryptographic signing operations,
ensuring transaction security and validating requests before granting
signatures locally.

Start a local signer and attach it to Greenlight by running:
```bash
glcli signer run
```

The signer now listens for incomming requests.

### Operate a Greenlight Node

_(Optional scheduling)_: When executing a `node` command, `glcli` will
automatically start the node by calling Greenlight's scheduler if necessary. 
However, you can manually schedule your node in advance by running:
```bash
glcli scheduler schedule
```

Once provisioned, you can interact with your node using `node` subcommands. For
example to check the node's operational state:
```bash
glcli node getinfo
```

## Advanced Bitcoin Network Configuration

Greenlight supports running nodes on the `bitcoin` and `signet` networks, 
defaulting to `bitcoin`. To register a Greenlight node on `signet`, use the 
`--network` option:
```bash
glcli --network="signet" scheduler register
```
Include the `--network` option in all susequent commands for this node. For
example:
```bash
glcli --network="signet" signer run
```
```bash
glcli --network="signet" node getinfo
```

_Please note that we __do not__ support `testnet` at the moment. Please open an
issue describing your use case if you need support for other networks._ 

## Development
### Build `glcli`
```bash
cargo build
```
### Run Tests
```bash
cargo test
```

## Contributing
`glcli` is under active development and currently lacks many major commands from
[Core-Lightning]("https://github.com/ElementsProject/lightning") such as
`listinvoices`, `fundchannel`, `close` and `sendpay`. Contributions to
add these and other missing commands are welcome. If you need a command 
that is not yet available, feel free to submit a pull request or open an issue 
describing your use case.

If you encounter any bugs, please report them via an issue or contribute a fix
through a pull request.

## License
`glcli` is licensed under the __MIT License__
