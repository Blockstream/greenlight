# `gl-testing`: A testing framework for Greenlight

`gl-testing` aims to facilitate local testing of a) Greenlight
integrations, b) the client libraries themselves and c) the signer
logic. It is based on `pytest` and `pyln-testing` and allows creating
reproducible Lightning Network setups backed by a local `bitcoind`
running `regtest` and includes a mock Greenlight service. When running
in `gl-testing` the client libraries are configured to talk to the
local scheduler, and nodes are spawned locally.

Each test scenario runs in isolation, with their own certificates,
their own scheduler and their own nodes. Tests can be run in parallel
since each scheduler will be using a different port, and tests are
confined to their own directory tree.

`gl-testing` faithfully recreates the external interface of
Greenlight, but is simplified and may omit some details.

## Testing

It is strongly suggested to run tests in a `docker` container, using
the image created from the `Dockerfile` in this directory. This is due
to the number of dependencies and configuration variables required to
correctly setup a testing environment, and also helps avoiding issues
with different platforms, where some of the dependencies might not be
present.

To build the docker container please run the following from the
root of this repository directory:

```bash
sudo make docker-image
```

This will create a docker container will dependencies installed and
pre-configured for testing.

### Testing integrations

In order to write a test create a new python file whose name begins
with `test_`, this will enable `pytest` to enumerate them and run
them. `gl-testing` and `pyln-testing` provide a large number of
fixtures used to initialize and interact with the environment in the
tests. For details refer to their respective APIs.

The following is a simple test scenario, in which we start the
Greenlight scheduler, register a new node using a client, schedule the
node to be run and then interact with the node using the client
libraries:

```python3
from gltesting.fixtures import *
from rich.pretty import pprint


def test_node_connect(scheduler, clients):
    """Register and schedule a node, then connect to it.
    """
    c = clients.new()
    c.register(configure=True)
    n = c.node()
    info = n.get_info()
    pprint(info)
```

The function represents a single test scenario, and uses the
`scheduler` and `clients` fixtures as arguments. These represent fully
initialized components of the system and can be used to interact with
the scheduler through its admin interface, and create clients
respectively.

While `c` created from `clients` is a fully configured client with its
own secret, signer and client library, developers will mostly likely
want to replace that with calls to their own app or library. Any
python code, or subprocess, will inherit the configuration to talk to
the mock scheduler instead of the production scheduler. This is
achieved by setting an environment variable, and allows interaction
with external tooling and binaries under test.

To run this test use the `docker` image above by running:

```bash
sudo docker run -ti --rm -v /your/code/directory:/repo gltesting pytest /path/to/tests
```

This will start the container, mount your code directory into it, and
then run `pytest` which should enumerate and run any tests you have
written.

The following options to `pytest` might be useful:

 - `--pdb`: When encountering an exception that is not being handled
   the execution interrupts and drops into an interactive python repl,
   allowing manual inspection and intervention. See the [pdb
   docs][pdb] for available commands.
 - `-v` Be more verbose about the execution. Add multiple times to
   further increase verbosity.
 - `-s`: Do not capture output from the test being run. This is useful
   to see what the various components are logging while running, and
   can be used for simple debugging. Even without this flag, captured
   output of failing tests will be printed at the end of the
   execution.
 - `-k <substr>`: Limit test execution to tests containing the
   specified `<substr>` in their name

### Testing signer / rust libraries

Since the signer and the rust libraries need to be built we will need
to get a shell in the docker container to compile the binaries and run
the tests. Tests for this case are provided in the `tests` directory,
so we can just mount the repository into the container, allowing us to
use any editor or tool for development outside the container, while
compiling and testing inside.

```bash
sudo make docker-shell
```

This will provide a `bash` shell in the container, with the repository
mounted into `/repo`. Any changes outside the container will be
reflected inside the container and vice-versa.

After making changes to the signer or libraries recompile and
reinstall the python bindings, so we can run the tests:

```bash
make build-self check-self
```

This will build the Rust libraries, the python bindings 

### Testing client bindings

This self-test mode exercises the `gl-client`, `gl-client-py` and
`gl-testing` libraries (other language bindings should be added in
future).

### Standalone scheduler

TBD

## Differences between Greenlight and `gl-testing`

 - Nodes are not preempted after a predetermined timeout. They are
   scheduled on the first `Scheduler.schedule` call and will run until
   the test completes.

## Local Setup

### Dependencies

The following dependencies and binaries must be installed, and
binaries must be on the `$PATH`.

 - [`cfssl`][cfssl]: used to generate mTLS private-keys and
   certificates used throughout Greenlight for authentication.
 - CLN versions (v0.10.1, v0.10.2, v0.11.01, v0.11.2gl2), adding the
   directories separated by `:` to the `CLN_PATH` environment variable
   so that `gl-testing` can find them. The directory should be the
   parent directory of `bin/`.


[pdb]: https://docs.python.org/3/library/pdb.html
[cfssl]: https://github.com/cloudflare/cfssl
