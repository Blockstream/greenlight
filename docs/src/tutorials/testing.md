# Testing with `gl-testing`

The [repository][repo] includes a testing framework that can be used
to test your application locally. This framework is built on top of
[pyln-testing][pyln-testing] (also used to develop Core Lightning
itself), allowing developers to describe arbitrarily complex network
setups, alongside a mock Greenlight service, and test their
functionality in a repeatable and reproducible way.

If you are already familiar with [pytest][pytest] you should feel
right at home, if not, don't worry, we will walk through an example
together here.

## Why test and what to test?

You have just written the next viral app on top of Greenlight, how do
you ensure it a) works now, and b) keeps on working going forward? You
could manually test your application after each change against the
Greenlight servers, or you might even automate some of these, but they
still run against the production environment. On one hand this is
likely not as fast as you're used to for local tests, and it actually
allocates resources on the service that someone will have to pay for.

Clearly it'd be better if we had a mock implementation of Greenluight
that you can run locally, and that can be torn down in order to free
the resources. This is pretty much what `gl-testing` is for. It
consists of a mock python implementation of the `Scheduler`, on top of
all the existing utilities from `pyln-testing`, and is bundled in a
simple to use docker image. The docker image comes with all the
dependencies installed, such as multiple CLN versions to test against,
and is pre-configured to create an environment that is as close to the
production setup as possible.

!!! example "Differences between `gl-testing` and the production environment"
	We keep track of when behavior of `gl-testing` diverges substantially from the production behavior in the [`gl-testing` readme][gl-testing-diff]

Testing in the docker images is optional for Linux hosts, but strongly
suggested due to the rather large number of dependencies. For Windows
and MacOS we only support testing in the docker image, since the
pre-compiled CLN versions are compiled for Linux on `x86_64` only for
now.

## Self-testing

You will probably have expected this, but we also use `gl-testing` to
test the bindings themselves. If you are working on a pull request for
`gl-client` or one of the other components it is strongly suggested
that you test your changes prior to submitting. To do so you can use a couple of `Makefile` targets:

```bash
# Create the `gltesting` docker image
make docker-image

# Open an interactive shell in an instance of `gltesting`
make docker-shell
```

Then from inside the `docker-shell` you can invoke some more commands:

```bash
# Build the `gl-client-py` bindings and install them in our python environment
make build-self

# Run the tests in `libs/gl-testing/tests` against the framework
make check-self
```

!!! warning
	Depending on you system setup you may need to prefix these commands
	with `sudo` or become root first. This is because `docker`, which is
	used by those `Makefile` targets requires the user to either be in the
	`docker` group, or be `root`.

## Writing your own test

Tests in `gl-testing` work best if you have a programmatic way of
driving your client. This could either be your own testing framework,
e.g., having a method to trigger a button press in your UI, or by
exposing your own API. In this example we will walk through a simple test that

 1. Sets up a small test network
 2. Starts a Greenlight node
 3. Opens a channel from the network to the greenlight node
 4. Creates an invoice on the greenlight node
 5. Pays the invoice from a node in the network

Here's the example code in its entirety and we will walk through the individual parts afterwards:

```python linenums="1"
def test_node_network(node_factory, clients, bitcoind):
	"""Setup a small network and check that we can send/receive payments.

	"""
	l1, l2 = node_factory.line_graph(2)

	c = clients.new()
	c.register(configure=True)
	gl1 = c.node()

	# Handshake needs signer for ECDH of Noise_XK exchange
	s = c.signer().run_in_thread()
	gl1.connect_peer(l2.info['id'], f'127.0.0.1:{l2.daemon.port}')

	# Now open a channel from l2 -> gl1
	l2.fundwallet(sats=2*10**6)
	l2.rpc.fundchannel(c.node_id.hex(), 'all')
	bitcoind.generate_block(1, wait_for_mempool=1)

	# Now wait for the channel to confirm
	wait_for(lambda: gl1.list_peers().peers[0].channels[0].state == 'CHANNELD_NORMAL')

	inv = gl1.create_invoice(
		'test',
		nodepb.Amount(millisatoshi=10000)
	).bolt11

	l1.rpc.pay(inv)
```

Line 1: we define a new test:

```py linenums="1"
def test_node_network(node_factory, clients, bitcoind):
```

Any function that starts with `test_` will be picked up by the test
runner and executed. The arguments are fixtures that can be
users. More on fixtures further down.


Next we create a small network of two nodes, `l1` and `l2`, connected
by a channel using the `node_factory` fixture. This fixture is used to
start and control non-greenlight nodes.

```py linenums="5"
	l1, l2 = node_factory.line_graph(2)
```

Using the `clients` fixture we create a new client, along with its own
directory, signer secret, and certificates. We then use
`Client.register()` to register the node with Greenlight and then
schedule it right away with the call to `Client.node()`

```py linenums="7"
	c = clients.new()
	c.register(configure=True)
	gl1 = c.node()
```

Lines 14-21: We start the signer (required to complete the handshake
and any other signer request we will encounter), connect outwards from
the GL node to `l2` representing the rest of the network, and then
using `l2` to fund a channel over the connection we just opened.

```py linenums="11"
	# Handshake needs signer for ECDH of Noise_XK exchange
	s = c.signer().run_in_thread()
	gl1.connect_peer(l2.info['id'], f'127.0.0.1:{l2.daemon.port}')


	# Now open a channel from l2 -> gl1
	l2.fundwallet(sats=2*10**6)
	l2.rpc.fundchannel(c.node_id.hex(), 'all')
	bitcoind.generate_block(1, wait_for_mempool=1)
```

Notice that the last step waits for the funding transaction and
confirms it. We then need to wait for the two nodes to notice that the funding transaction was confirmed before continuing:

```py linenums="20"
	# Now wait for the channel to confirm
	wait_for(lambda: gl1.list_peers().peers[0].channels[0].state == 'CHANNELD_NORMAL')
```

This will poll `gl1` for its channel states and return as soon as the
state indicates that the channel is confirmed and fully functional.

Finally we can create an invoice from the Greenlight node `gl1` and
pay it using `l1`, which should result in a path `l1 -> l2 -> gl1`
being used, and the invoice will be paid after this:

```py linenums="23"
	inv = gl1.create_invoice(
		'test',
		nodepb.Amount(millisatoshi=10000)
	).bolt11

	l1.rpc.pay(inv)
```

To run this test you can use the above `Makefile` target:

```bash
$ sudo make docker-image docker-shell  # (1)!
$ make build-self check-self  # (2)!
```

 1. Enter the docker shell, building the image if it was not generated before.
 2. In the docker shell, build `gl-client` and bindings, then execute
    the self-tests

If you can drive your application from a python script through a UI
testing framework or an API, you can just use the above test as a
template and start writing your own tests.

If you don't have a way to run your application through a scripted
test you may want to start a network, and test against that
manually. For an example of this see the next section.

## Manually testing against a mock Network

Every once in a while you'll want to either step through an existing
test, or have a small test that just sets up a network topology, and
then drops you in a shell that you can use to interact with the
network. In both cases `breakpoint()` is your friend

The following test will set up the small network above, and then drop
you in a REPL that you can use to inspect the setup, and to drive
changes such as paying an invoice or funding a channel:

```py
from gltesting.fixtures import *
import os


def test_my_network(clients, node_factory, scheduler, directory, bitcoind):
	"""Start a small line_graph network to play with.
	"""
	l1, l2, l3 = node_factory.line_graph(3)  # (1)!

	# Assuming we want interact with l3 we'll want to print
	# its contact details:
	print(f"scheduler: https://localhost:{scheduler.grpc_port}")  # (3)!
	print(f"l3 details: {l3.info['id']} @ 127.0.0.1:{l3.daemon.port}")

	print(f"CA Cert:     {directory}/certs/ca.pem")  # (4)
	print(f"Nobody Cert: {directory}/certs/users/nobody.pem")
	print(f"Nobody Key:  {directory}/certs/users/nobody-key.pem")

	breakpoint()  # (2)!
```

 1. At this point we have a network with 3 nodes in a line.
 2. Opens a REPL that accepts Python code.
 3. Tells us which port the mock scheduler is listening on
 4. Prints the location of the keypairs and certificates to use when
    talking to the mock scheduler

To start this test just run:

```bash
$ pytest testy.py -s
========== test session starts ==========
platform linux -- Python 3.8.10, pytest-7.2.1, pluggy-1.0.0
rootdir: /repo
plugins: cov-3.0.0, xdist-2.5.0, forked-1.6.0, timeout-2.1.0
collected 1 item

testy.py Running tests in /tmp/ltests-syfsnw83
[... many more lines about the setup of the network ...]

scheduler: https://localhost:43841
l3 details: 035d2b1192dfba134e10e540875d366ebc8bc353d5aa766b80c090b39c3a5d885d @ 127.0.0.1:34547
CA Cert:     /tmp/gltesting/tmpfo6c2ye2/certs/ca.pem
Nobody Cert: /tmp/gltesting/tmpfo6c2ye2/certs/users/nobody.pem
Nobody Key:  /tmp/gltesting/tmpfo6c2ye2/certs/users/nobody-key.pem

>>>>>>>>>> PDB set_trace >>>>>>>>>>
--Return--
> /repo/testy.py(20)test_my_network()->None
-> breakpoint()
(pdb)
```

At this point you will have a REPL that you can use to drive changes,
by writing python code, just like you'd do if you were writing the
test in a file.

In order to attach your client application to the mock scheduler we
need to set a number of environment variables that `gl-client` will
pick up and use:

```bash
export GL_NOBODY_CERT=/tmp/gltesting/tmpb7711nx5/certs/users/nobody.crt
export GL_NOBODY_KEY=/tmp/gltesting/tmpb7711nx5/certs/users/nobody-key.pem
export GL_CA_CRT=/tmp/gltesting/tmpb7711nx5/certs/ca.pem

export GL_SCHEDULER_GRPC_URI=http://127.0.0.1:43841
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

All of this works thanks to us mounting the `/tmp/gltesting` directory
from the host, allowing both docker container and host to exchange
files, and `docker-shell` also reuses the host network, allowing
clients running on the host to talk directly to the scheduler and
nodes, without having to change the IP and setup port-forwarding.

Once you are done testing, use `continue` or `Ctrl-D` in the REPL to
trigger a shutdown.

## Fixtures

TODO

[repo]: https://github.com/Blockstream/greenlight
[pyln-testing]: https://github.com/ElementsProject/lightning/tree/master/contrib/pyln-testing
[pytest]: https://docs.pytest.org/en/7.2.x/
[gl-testing-diff]: https://github.com/Blockstream/greenlight/tree/main/libs/gl-testing#differences-between-greenlight-and-gl-testing
