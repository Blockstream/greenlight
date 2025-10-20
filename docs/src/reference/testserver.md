# The `gl-testserver`

The `gl-testserver` is a standalone version, of the `gl-testing`
framework, allowing you to test against a mock Greenlight server,
independently of your programming language and development
environment.

The goal of the `gl-testing` package is to enable developers to test
locally against a mock Greenlight server. This has a number of
advantages:

 - **Speed**: by not involving the network, you can test without
   latency slowing you down. The tests also run on a `regtest`
   network, allowing you to generate blocks and confirm transactions
   without the need to wait.
 - **Cost**: the `prod` network is not free, and since tests may
   consume arbitrary resources, which are then not cleaned up (see
   next point), repeatedly running them could incur costs. We see this
   as a bad incentive to minimize testing during development, and
   `gl-testing` allows you to use only local resources that can then
   also be reclaimed, making testing free, and hopefully encouraging
   to test more.
 - **Reproducibility**: The `prod` network does not allow cleaning up
   test resources, since there might be an actual user using
   them. This means that test artifacts persist between runs,
   resulting in a non-reproducible environment for
   testing. `gl-testing` running locally allows cleaning up resources,
   thus enabling reproducible tests.

However, the downside of `gl-testing` is that is coupled with `python`
as programming language and `pytest` as test runner. This is where
`gl-testserver` comes in: by bundling all the fixtures and startup
logic into a standalone binary we can pull up an instance in a matter
of seconds, test and develop against it, and then tear it down at the
end of our session.

## How to use `gl-testserver`

It's probably easiest to use `uv` to run the script from the source
tree. Please see the [`uv` installation instructions][uv/install] for
how to get started installing `uv` then come back here.

Executing `uv run gltestserver` is the entrypoint for the tool:

```bash
gltestserver
Usage: gltestserver [OPTIONS] COMMAND [ARGS]...

Options:
  --help  Show this message and exit.

Commands:
  run  Start a gl-testserver instance to test against.
```

Currently there is only the `run` subcommand which will start the
testserver, binding the scheduler GRPC interface, the `bitcoind` RPC
interface, and the GRPC-web proxy to random ports on `localhost`.


```bash
gltestserver run --help
Usage: gltestserver run [OPTIONS]

  Start a gl-testserver instance to test against.

Options:
  --directory PATH  Set the top-level directory for the testserver. This can
					be used to run multiple instances isolated from each
					other, by giving each isntance a different top-level
					directory. Defaults to '/tmp/'
  --help            Show this message and exit
```

In order to identify the ports for the current instance you can either
see the end of the output of the command, which contains
pretty-printed key-value pairs, or load the `metadata.json` file
containing the port references to use from the `gl-testserver`
sub-directory (`/tmp/gl-testserver` if you haven't specified the
`--directory` option).

!!! note "Running multiple tests in parallel"
	As the help text above already points out it is possible to run as many
	instances of the testserver concurrently as you want, by specifying
	separate `--directory` options in each call.

	This is particularly useful if you want to run multiple tests in parallel
	to speed up the test runs. It is also the core reason why ports are
	randomized rather than using fixed ports per interface, as concurrent
	instances would conflict, and the isolation between tests could be
	compromised.

Once started you will see the following lines printed:

```
Writing testserver metadata to /tmp/gl-testserver/metadata.json
{
  'scheduler_grpc_uri': 'https://localhost:38209',
  'grpc_web_proxy_uri': 'http://localhost:35911',
  'bitcoind_rpc_uri': 'http://rpcuser:rpcpass@localhost:44135'
}
Server is up and running with the above config values. To stop press Ctrl-C.
```

At this point you can use the URIs that are printed to interact with
the services, or use `Ctrl-C` to stop the server. When running in a
test environment you can send `SIGTERM` to the process and it'll also
shut down gracefully, cleaning up the processes, but leaving the data
created during the test in the directory.

[uv/install]: https://docs.astral.sh/uv/getting-started/installation/
