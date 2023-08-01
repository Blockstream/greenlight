# Testing your Application

!!! example "In this tutorial you will learn how to:"
	* Build and use the testing [Docker][docker] image
    * Set up a test network with Core Lightning nodes
	* Use the Greenlight test framework `gl-testing`
	* Write a simple test that utilizes a Greenlight node
	* Start a python REPL that lets you manually test against `gl-testing`
## Why test and what to test?

You have just written the next viral app on top of Greenlight, how do
you ensure it

 1. works now?
 2. keeps on working going forward?

You could manually test your application after each change against the
Greenlight servers, or you might even automate some of these, but they
still run against the production environment. This is likely not as 
fast as you're used to for local tests, and it actually allocates 
resources on the Greenlight service that someone will have to pay for.

It would undoubtedly be better if we had a way to test our application
locally and in reproducible way. Well, fortunately, Greenlight 
provides a testing framework that helps you do just that.

## The `gl-testing` testing framework

The Greenlight [repository][repo] comes with a "Batteries Included" 
testing framework that you can use to test your application locally.
The testing framework `gl-testing` is based on [pyln-testing][pyln-testing]
which is also used in the development of Core Lightning itself.

The components of `gl-testing` allow you to:

- Construct an arbitrarily complex network of lightning nodes.
- Set up a local mock of the Greenlight services.
- Use the provided pyln-fixtures for sophisticated test setups.
- Test your application in a repeatable and reproducible manner.
- Keep you system clean of development dependencies.

!!! info "Deviations in behavior between `gl-testing` and the production environment"
	We keep track of the substantial differences in the behavior of 
	`gl-testing` and the production system in the 
	[`gl-testing` readme][gl-testing-diff]


This tutorial will walk you through the necessary steps to write a 
test that registers a new Greenlight client with the `gl-testing` 
testing framework that issues an invoice from the newly registered 
Greenlight node.
You will also learn how to start a REPL that you can use to manually
execute commands against the testing framework or your application
## Prerequisites
#### Git
The `gl-testing` testing framework is part of the Greenlight github
[repository][repo]. To get a local working copy of the Greenlight
repository you need `git` installed on your system. See the 
[git-guides][git-install] for a detailed instruction on how to install
`git` on your system.

#### protoc
The later section `Manually testing against a mock Network` requires us
to build the `gl-client` on the host system. Greenlight uses [grpc][grpc]
for the communication between the client and the node, therefor you need 
the Protobuf compiler `protoc` present on our machine. Check out
[protoc][protoc] for instructions on how to install `protoc` on your
system.

#### Docker
Testing a Greenlight application is dependency intensive. We need 
different versions of Core Lightning to be present besides a bunch of
python packages, rust and cargo, as well as a compiler for proto 
files. To help you keep your development environment clean, the 
`gl-testing` testing framework comes with a __Dockerfile__ that includes 
all the necessary dependencies and allows you to run all the tests in 
the shell of the assembled Docker image.

You need a working Docker installation on your system in order to 
build and use the Docker image. See the [Docker manual][docker-install]
for instructions on how to set up Docker on your operating system.

!!! tip
	Testing in the docker images is optional for Linux hosts, but strongly
	suggested due to the rather large number of dependencies. For Windows
	and MacOS we only support testing in the docker image, since 
	Core Lightning only ships pre-compiled versions for Linux on 
	`x86_64` for now.

## Prepare your local environment
Before we can dive into testing with the `gl-testing` testing
framework we need to get a local working copy of the repository.

``` { .bash .copy}
git clone git@github.com:Blockstream/greenlight.git gl-testing-tutorial
```

For the rest of the tutorial we will work within the repository
we just cloned.
``` { .bash .copy}
cd gl-testing-tutorial
```

The Greenlight repository comes with a `Makefile` that holds some 
useful targets for us. We make use of this to build a Docker image 
`gltesting` that contains all the dependencies required to run the 
testing framework `gl-testing`.
``` { .bash .copy}
make docker-image
```

Now we are all set and to drop into a shell that hold all the 
required dependencies to work with `gl-testing`.
``` { .bash .copy}
make docker-shell
```
You can always exit the docker-shell by calling `exit` from the shell 
or by pressing `Ctrl-D`.

!!! info "Self testing"
	You will probably have expected this, but we also use `gl-testing`
	to test the `gl-client` bindings themselves. If you are working on
	a pull request for the `gl-client` or another component, 
	`gl-testing` allows you to test your changes locally before 
	submitting them.

## Write your first test


Tests in `gl-testing` work best if you have a programmatic way of
driving your client. This could either be your own testing framework,
e.g., having a method to trigger a button press in your UI, or by
exposing your own API. In this example we will walk through a simple 
test that

 1. Sets up a small test network
 2. Starts a Greenlight node
 3. Opens a channel from the network to the Greenlight node
 4. Creates an invoice on the Greenlight node
 5. Pays the invoice from a node in the network


``` mermaid
flowchart LR
	A((CLN 1)) === B((CLN 2));
	B === C((GL 1));
	A -. payment .-> C;
```

We start by creating our test file `my-first-test.py` in the root of 
our `gl-testing-tutorial` directory. The `gl-testing` testing 
framework uses the [`pytest`][pytest] framework under the hood, so writing test
should be familiar for the python developers amongst you.

```py linenums="1"  title="my-first-test.py"
from gltesting.fixtures import *

def test_invoice_payment(node_factory, clients, bitcoind):
    print("Hello World!")
```

Here we import our test fixtures and create a simple test that just
prints `"Hello World!"` to the standard output. Any function that 
starts with `test_` will be picked up by the test runner and executed.
The arguments are fixtures (see [pytest][pytest] for further details)
that are passed to and can be used by the test.

Let's check if we can properly import the `gl-testing` fixtures in our
test. To run the test we need to drop to the shell of the Docker
image we created above.

```bash
make docker-shell
```

Before we can run any tests that require parts of Greenlight, such as 
the `gl-client`, the `gl-plugin` or any of the bindings, we need to
build those components from the Docker shell.

```bash
make build-self
```

In the shell we execute the `pytest` command to run the test. We add
the flags `-v` for a verbose output and `-s` to print all output to
the console instead of capturing it.

```
pytest -v -s my-first-test.py
```

This should produce a lot of output, with the last few lines reading 
something along the lines of
```{.bash .no-copy}
Hello World!
PASSED
BitcoinRpcProxy shut down after processing 0 requests
Calling stop with arguments ()
Result for stop call: Bitcoin Core stopping


============================================================ 1 passed in 4.10s =============================================================
```

Great! We have written our very first test. However, our test is still
fairly useless, let's replace it with something actually meaningful.

As a first step, we create a small network of Core Lightning nodes to 
which we will connect our Greenlight node later on.
Fortunately, `pyln-testing` provides us with some fixtures that handle
common tasks for us. We can use this fixture to start and control 
non-Greenlight nodes.

```py linenums="1" hl_lines="4-6"  title="my-first-test.py"
from gltesting.fixtures import *

def test_invoice_payment(node_factory, clients, bitcoind):
	# Create 2-node-network (l1)----(l2)
    l1, l2 = node_factory.line_graph(2)
	l2.fundwallet(sats=2*10**6)
```
Here we used the `node_factory` fixture to create a `line-graph` 
network consisting of two Core Lightning nodes `l1` and `l2` that 
already have a channel established between them. The nodes have an 
integrated rpc client that we can use to fund the `l2` node with 
`2000000sat`.

!!! tip "Tip: Use your IDEs autocompletion"
	If you want to use the __autocompletion__ features of your IDE you
	need to select the python interpreter form the environment set by 
	poetry in `libs/gl-testing`. You can then import the classes from 
	the fixtures and annotate the fixtures with its types.
	e.g. 
	``` { .python .no-copy }
	from gltesting.fixtures import *
	from gltesting.fixtures import Clients
	from pyln.testing.fixtures import NodeFactory, LightningNode
	...
	def test_xyz(node_factory: NodeFactory, clients: Clients):
	    nodes: list[LightningNode] = node_factory.line_graph(2)
	    l1, l2 = nodes[0], nodes[1]
	...
	```

Now we can finally start to deal with Greenlight. We use the `clients` 
fixture to create a new client, along with its own directory, signer 
secret, and certificates. After that we can call the 
`Client.register()` method to register the client with Greenlight and
and `Client.node()` to schedule and return the Greenlight node that 
belongs to the registered client. The `configure=True` argument tells
the client to store the client certificates.

```py linenums="1" hl_lines="8-11"  title="my-first-test.py"
from gltesting.fixtures import *

def test_invoice_payment(node_factory, clients, bitcoind):
    # Create 2-node-network (l1)----(l2)
    l1, l2 = node_factory.line_graph(2)
	l2.fundwallet(sats=2*10**6)

	# Register a new Greenlight client.
	c = clients.new()
    c.register(configure=True)
    gl1 = c.node()
```

__Congratulations__, we have our first Greenlight node up and running on
the testing framework! 

Let's connect our Greenlight node to the 
network now. To do so we need to establish a channel between our
Greenlight node and the network. We choose the `l2` node as the entry
point for our Greenlight node. Funding the channel between `l2` and 
`gl` requires us to connect to `l2` and to fund a channel. The 
connection handshake as well as the channel funding and eventually the
creation of an invoice require the presence of a signer for the node.
Greenlight signers run on the client side to keep the custody on the
users side. We first need to start the client signer so that the node
can request signatures from the signer.

```py linenums="1" hl_lines="13-15"  title="my-first-test.py"
from gltesting.fixtures import *

def test_invoice_payment(node_factory, clients, bitcoind):
	# Create 2-node-network (l1)----(l2)
    l1, l2 = node_factory.line_graph(2)
	l2.fundwallet(sats=2*10**6)

	# Register a new Greenlight client.
	c = clients.new()
    c.register(configure=True)
    gl1 = c.node()

	# Start signer and connect to (l2)
	s = c.signer().run_in_thread()
    gl1.connect_peer(l2.info['id'], f'127.0.0.1:{l2.daemon.port}')
```

We are almost there! Now we fund a channel between `l2` and `gl`. We 
import a helper function 
`:::py3 from pyln.testing.utils import wait_for` that helps us to wait
for the channel to be established.This will poll `gl1` for its channel
states and return as soon as the state indicates that the channel is 
confirmed and fully functional.

```py linenums="1" hl_lines="2 18-28"  title="my-first-test.py"
from gltesting.fixtures import *
from pyln.testing.utils import wait_for

def test_invoice_payment(node_factory, clients, bitcoind):
	# Create 2-node-network (l1)----(l2)
    l1, l2 = node_factory.line_graph(2)
    l2.fundwallet(sats=2*10**6)

	# Register a new Greenlight client.
    c = clients.new()
    c.register(configure=True)
    gl1 = c.node()

	# Start signer and connect to (l2)
    s = c.signer().run_in_thread()
    gl1.connect_peer(l2.info['id'], f'127.0.0.1:{l2.daemon.port}')

	# Fund a channel (l2)----(gl).
	# This results in the following network
	# (l1)----(l2)----(gl)
    l2.rpc.fundchannel(c.node_id.hex(), 'all')
	# Generate a block to synchronize and proceed
	# with the channel funding.
    bitcoind.generate_block(1, wait_for_mempool=1)
	# Wait for the channel to confirm.
    wait_for(lambda:
        gl1.list_peers().peers[0].channels[0].state == 'CHANNELD_NORMAL'
    )
```

Before we create and pay the invoice we also need to wait for the 
gossip to reach every node. Otherwise the invoice will be lacking
route hints as it assumes its only channel to be a dead end.
Alternatively we could also create the invoice directly and wait for 
`l1` to have a full view of our small network but lets go with the 
first option this time. We again use the `wait_for` function. The
successor function checks that we see 4 channel entries in our view
of the network as both channels are bidirectional.

```py linenums="1" hl_lines="30-38"  title="my-first-test.py"
from gltesting.fixtures import *
from pyln.testing.utils import wait_for

def test_invoice_payment(node_factory, clients, bitcoind):
	# Create 2-node-network (l1)----(l2)
    l1, l2 = node_factory.line_graph(2)
    l2.fundwallet(sats=2*10**6)

	# Register a new Greenlight client.
    c = clients.new()
    c.register(configure=True)
    gl1 = c.node()

	# Start signer and connect to (l2)
    s = c.signer().run_in_thread()
    gl1.connect_peer(l2.info['id'], f'127.0.0.1:{l2.daemon.port}')

	# Fund a channel (l2)----(gl).
	# This results in the following network
	# (l1)----(l2)----(gl)
    l2.rpc.fundchannel(c.node_id.hex(), 'all')
	# Generate a block to synchronize and proceed
	# with the channel funding.
    bitcoind.generate_block(1, wait_for_mempool=1)
	# Wait for the channel to confirm.
    wait_for(lambda:
        gl1.list_peers().peers[0].channels[0].state == 'CHANNELD_NORMAL'
    )

	# Wait for all channels to appear in our view of the network. We 
    # don't even have to wait for our channel to appear in l1s 
	# gossip: We can give a hint as soon as we know that our channel 
	# is not a dead end. We wait for 4 entries in our gossmap as both 
    # channels are bidirectional.
    bitcoind.generate_block(5)
    wait_for(
        lambda: len([c for c in gl1.list_channels().channels]) == 4 
    )
```

Now we can finally create and pay an invoice. We create an invoice on
the Greenlight node and pay it with the `l1` node routed via the `l2`
node. To create the invoice we import the core-lightning proto stubs
`clnpb` from the Greenlight client `glclient`.

```py linenums="1" hl_lines="3 41-49"  title="my-first-test.py"
from gltesting.fixtures import *
from pyln.testing.utils import wait_for
from glclient import clnpb

def test_invoice_payment(node_factory, clients, bitcoind):
	# Create 2-node-network (l1)----(l2)
    l1, l2 = node_factory.line_graph(2)
    l2.fundwallet(sats=2*10**6)

	# Register a new Greenlight client.
    c = clients.new()
    c.register(configure=True)
    gl1 = c.node()

	# Start signer and connect to (l2)
    s = c.signer().run_in_thread()
    gl1.connect_peer(l2.info['id'], f'127.0.0.1:{l2.daemon.port}')

	# Fund a channel (l2)----(gl).
	# This results in the following network
	# (l1)----(l2)----(gl)
    l2.rpc.fundchannel(c.node_id.hex(), 'all')
	# Generate a block to synchronize and proceed
	# with the channel funding.
    bitcoind.generate_block(1, wait_for_mempool=1)
	# Wait for the channel to confirm.
	wait_for(lambda:
        gl1.list_peers().peers[0].channels[0].state == 'CHANNELD_NORMAL'
    )

    # Wait for all channels to appear in our view of the network. We 
    # don't even have to wait for our channel to appear at the gossmap
    # of l1: We can give a hint as soon as we know that our channel is 
    # not a dead end. We wait for 4 entries in our gossmap as both 
    # channels are bidirectional.
    bitcoind.generate_block(5)
    wait_for(
        lambda: len([c for c in gl1.list_channels().channels]) == 4 
    )
   
    # Create an invoice.
    bolt11 = gl1.invoice(
        amount_msat=clnpb.AmountOrAny(amount=clnpb.Amount(msat=100000)),
	    description="test-desc",
	    label ="test-label",
    ).bolt11

    # Pay invoice.
    l1.rpc.pay(bolt11)
```

__Congratulations__! You have written your first test using the 
`gl-testing` testing framework and a Greenlight node. Our test creates
a small line graph network consisting of 3 lightning nodes with a 
Greenlight node at the end. We created an invoice on the Greenlight
node and payed for it with the first node in line.

Let's check if our test passes!

Remember that we need to call the tests from our docker shell
```bash 
make docker-shell
```

We can start our `pytest` test now. Remember the options `-v` and `-s`
to print the logs to stdout instead of capturing them. 
```bash
pytest -vs my-first-test.py
```

After the test is finished you should see `passed` in the terminal.
```bash
========================================== 1 passed in 30.83s ===========================================
```

## Manually testing against a mock Network

Every once in a while you'll want to either step through an existing
test, or have a small test that just sets up a network topology, and
then drops you in a shell that you can use to interact with the
network. In both cases `breakpoint()` is your friend.

You also can use a `breakpoint()` to set up a `gltesting` environment
that you can work against from your host.

The following test will set up a small lightning network, and then 
drops us in a [REPL][repl] that we can use to inspect the setup, and to 
drive changes such as paying an invoice or funding a channel. Note 
that we also import and pass the `scheduler` `directory` fixtures to
the test so that we can access them from our REPL:

```py linenums="1" title="examples/setup_repl.py"
from gltesting.fixtures import *

def test_setup(clients, node_factory, scheduler, directory, bitcoind):
    """Sets up a gltesting backend and a small lightning network.

    This is meant to be run from inside the docker shell. See the 
    gltesting tutorial for further info.
    """
    l1, l2, l3 = node_factory.line_graph(3)  # (1)!
	
    # Assuming we want interact with l3 we'll want to print
	# its contact details:
    print(f"l3 details: {l3.info['id']} @ 127.0.0.1:{l3.daemon.port}")
    print()
    print(f"export GL_CA_CRT={directory}/certs/ca.pem")  # (4)
    print(f"export GL_NOBODY_CRT={directory}/certs/users/nobody.crt")
    print(f"export GL_NOBODY_KEY={directory}/certs/users/nobody-key.pem")
    print(f"export GL_SCHEDULER_GRPC_URI=https://localhost:{scheduler.grpc_port}")  # (3)!
	
    breakpoint()  # (2)!
```

 1. At this point we have a network with 3 nodes in a line.
 2. Opens a REPL that accepts Python code.
 3. Tells us which port the mock scheduler is listening on
 4. Prints the location of the keypairs and certificates to use when
    talking to the mock scheduler

To run this test we first need to drop into the Docker shell.
```bash
make docker-shell
```

Then we can start our REPL form inside the docker-shell.
```bash
pytest -s examples/setup_repl.py
```

You will see an output that looks similar to the following lines:
```bash
$ pytest -s testy.py
========== test session starts ==========
platform linux -- Python 3.8.10, pytest-7.2.1, pluggy-1.0.0
rootdir: /repo
plugins: cov-3.0.0, xdist-2.5.0, forked-1.6.0, timeout-2.1.0
collected 1 item

testy.py Running tests in /tmp/ltests-syfsnw83
[... many more lines about the setup of the network ...]

scheduler: https://localhost:44165
l3 details: **node_id** @ 127.0.0.1:40261
export GL_CA_CRT=/tmp/gltesting/**tmpdir**/certs/ca.pem
export GL_NOBODY_CRT=/tmp/gltesting/**tmpdir**/certs/users/nobody.crt
export GL_NOBODY_KEY=/tmp/gltesting/**tmpdir**/certs/users/nobody-key.pem
export GL_SCHEDULER_GRPC_URI=https://localhost:**scheduler_port**

>>>>>>>>>> PDB set_trace >>>>>>>>>>
--Return--
> /repo/testy.py(20)test_my_network()->None
-> breakpoint()
(pdb)
```

At this point we have a REPL that we can use to drive changes 
interactively, by writing python code, just like we'd do if we were 
writing the test in a file.

We now want to attach a client application from the host to the mock 
scheduler. Therefore we first need to set a number of environment 
variables on our host, that the `gl-client` library will pick up and 
use. Just copy the following lines from your docker-shell:

```bash
export GL_CA_CRT=/tmp/gltesting/**tmpdir**/certs/ca.pem
export GL_NOBODY_CRT=/tmp/gltesting/**tmpdir**/certs/users/nobody.crt
export GL_NOBODY_KEY=/tmp/gltesting/**tmpdir**/certs/users/nobody-key.pem
export GL_SCHEDULER_GRPC_URI=https://localhost:**scheduler_port**
```

The first three lines tell the client library which identity to load
itself, and how to verify the identity of the scheduler when
connecting. These must match the lines printed above. The last line
tells the client to connect to our mock scheduler instead of the
production scheduler, the port must match the one printed above.

!!! question "Why is this random?"
	We usually run tests in parallel, which requires that we isolate
	the tests from each other. If we did not randomize the ports and
	directories, we could end up with tests that interfere with each
	other, making debugging much harder, and resulting in flaky tests.

We now can create a client on our host that we can mess around with.

Lets have a look at the following example application that we will
explain in more detail in another tutorial. You can find the file
in the repository under `examples/app_test.py`.

```py linenums="1" title="examples/app_test.py"
import os
import pytest
from glclient import Scheduler, Signer, TlsConfig,Node, nodepb

class GetInfoApp:
    """An example application for gltesting.
    
    This example application shows the process on how to register,
    scheduler and call against a gltesting environment greenlight 
    node.

    To execute this example set up the docker gltesting environment,
    drop into a REPL as explained in the gltesting tutorial.

    Then run the test below outside the gltesting docker container
    (run it from the host).
    `pytest -s -v app_test.py::test_getinfoapp`.
    """
    def __init__(self, secret: bytes, network: str, tls: TlsConfig):
        self.secret: bytes = secret
        self.network = network
        self.tls: TlsConfig = tls
        self.signer: Signer = Signer(secret, network, tls) # signer needs to keep running
        self.node_id: bytes = self.signer.node_id()

    def scheduler(self) -> Scheduler:
        """Returns a glclient Scheduler

        The scheduler is created from the attributes stored in this
        class.
        """
        return Scheduler(self.node_id, self.network, self.tls)

    def register_or_recover(self):
        """Registers or recovers a node on gltesting
        
        Also sets the new identity after register/recover.
        """
        res = None
        try:
            res = self.scheduler().register(self.signer)
        except:
            res = self.scheduler().recover(self.signer)
        
        self.tls = self.tls.identity(res.device_cert, res.device_key)

    def get_info(self) -> nodepb.GetInfoResponse:
        """Requests getinfo on the gltesting greenlight node"""
        res = self.scheduler().schedule()
        node = Node(self.node_id, self.network, self.tls, res.grpc_uri)
        return node.get_info()


def test_getinfoapp():
    # These are normally persisted on disk and need to be loaded and
    # passed to the glclient library by the application. In this 
    # example we store them directly in the "app".
    secret = b'\x00'*32
    network='regtest'
    tls = TlsConfig()

    # Register a node
    giap = GetInfoApp(secret, network, tls)
    giap.register_or_recover()

    # GetInfo
    res = giap.get_info()
    print(f"res={res}")
```

This example application registers a node and requests `getinfo` on
the greenlight node. Let's check if we can run it agains our REPL
gltesting setup.

We need to switch to the example directory (on our host, not in the 
docker-shell) and activate the python environment that sets up all 
the necessary dependencies for the
example.

```bash
poetry shell
```

```bash
poetry install
```

The first command drops us into a [`poetry`][poetry]-shell, the second
installs the necessary dependencies from the `pyproject.toml` file.

With the REPL setup in the docker-shell and from the poetry-shell on 
the host we can now run our test application.

```bash
pytest -s app_test.py::test_getinfoapp
```

If you now see something an output that looks similar to the following
lines, you made it! You successfully set up a `gltesting` greenlight
mock in the `docker-shell` an application against it from the host.

```bash
================== test session starts ==================
...                                                        
app_test.py::test_getinfoapp res=id: "\002\005\216\213l*\323c\354Y\252\023d)%mtQd\302\275\310\177\230\360\246\206\220\354,\\\233\013"
alias: "VIOLENTSPAWN-v23.05gl1"
color: "\002\005\216"
version: "v23.05gl1"
lightning_dir: "/tmp/gltesting/tmp/tmpdz6neih7/node-0/regtest"
our_features {
  init: "\010\240\210\n\"i\242"
  node: "\210\240\210\n\"i\242"
  invoice: "\002\000\000\002\002A\000"
}
blockheight: 103
network: "regtest"
fees_collected_msat {
}

PASSED

================== 1 passed in 0.10s ==================

```

All of this works thanks because we mount the `/tmp/gltesting` directory
from the host, allowing both, the docker container and host to exchange
files. The `docker-shell` also reuses the host network, allowing
clients or applications running on the host to talk directly to the 
scheduler and the nodes running in the docker container.

Once you are done testing, use `continue` or `Ctrl-D` in the REPL to
trigger a shutdown.

[docker]: https://www.docker.com/
[docker-install]: https://docs.docker.com/engine/install/
[git-install]: https://github.com/git-guides/install-git
[repo]: https://github.com/Blockstream/greenlight
[pyln-testing]: https://github.com/ElementsProject/lightning/tree/master/contrib/pyln-testing
[pytest]: https://docs.pytest.org/en/7.2.x/
[gl-testing-diff]: https://github.com/Blockstream/greenlight/tree/main/libs/gl-testing#differences-between-greenlight-and-gl-testing
[stickers]: https://store.blockstream.com/product/sticker-bundle/
[repl]: https://en.wikipedia.org/wiki/Read%E2%80%93eval%E2%80%93print_loop
[poetry]: https://python-poetry.org
[grpc]: https://grpc.io/
[protoc]: https://grpc.io/docs/protoc-installation/