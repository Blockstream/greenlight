# glcli: Greenlight Command Line Interface

A simple tool to issue commands to a Greenlight node and interact with
the scheduler.

## Help

```bash
$glcli scheduler --help
```


```bash
$ glcli --help
```

## Installing

Installing the `glcli` utility can be done with the following command:

```bash
pip install --extra-index-url=https://us-west2-python.pkg.dev/c-lightning/greenlight-pypi/simple/ .
```

In most cases we have prebuilt the binary extension for `gl-client-py`
(which internall depends on `libhsmd`, another binary extension). If
you run any of the following platforms you'll get a precompiled
version:

 - MacOS ARM64 (cpython 3.6, 3.7m, 3.8, 3.9 and 3.10)
 - Linux glibc 2.24 amd64 and i686(cpython 3.6, 3.7m, 3.8, 3.9 and 3.10)

Should your platform not be among the precompiled versions you will
need to have some additional dependencies to build it on the fly.

### Ubuntu / Debian

The following dependencies are required to install from source.

```bash
apt-get update
apt-get install -y \
	python3-pip \
	clang \
	pkg-config \
	autoconf \
	libtool \
	git \
	libsqlite3-dev \
	libgmp-dev \
	curl

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain nightly -y
source $HOME/.cargo/env

pip3 install --user maturin
export PATH=$PATH:~/.local/bin
```

The `glclient` dependency will require at least rustc 1.56 since
that's when the `edition2021` feature was stabilized.
