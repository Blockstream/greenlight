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

Installation is a bit more complex than necessary at the moment. The
main issue is that installing `glclient` from a relative directory is
not supported by `pip`. The `Makefile` in this directory has a `make
install` target that can be used to install the packages in the
correct order in the currently active `virtualenv`.

```bash
make install
```


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
