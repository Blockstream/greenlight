# This dev-container is used to build and test greenlight.
# It can also be used to generate code such as the grpc-bindings.
#
# Typically, Dockerfiles are optimized for size.
# This allows quick deployments and pulls.
# However, this Dockerfile isn't deployed nor pushed to a registry.
# There is no need to optimize size for the time being. 
#
# I've decided to optimize for the following attributes
#
# Developer Experience:
# We want to mount the repository in the Dockerfile.
# I want to have editable installs where possible.
#
# The docker-file should be easy to run under your own user.
# This ensures we never have rights issues on our own system.
#
# Caching
# The Dockerfile should provide good caching
# We only want to rebuild parts that have actually changed.
# E.g: We will only reinstall python dependencies if
# a `pyproject.toml` file has changed
#
# Build-time
# Prefer binary installs over compilation where possible.
# We also heavily rely on staged builds to increase concurrency.

# --------------------------------------------------
# STAGE: python-builder
#--------------------------------------------------
# This imaged is used to build the python environment
# It gets all dependencies from our `pyproject.toml`
# files and installs them in a `venv`.
#
# Later, we'll copy this venv to our project
FROM ubuntu:22.04 AS python-builder
RUN apt update && apt install -qqy curl python3 python3-pip python3-venv libpq-dev

# Create a python-venv that will include all dependencies
ENV PATH=/tmp/venv/bin:$PATH
RUN python3 -m venv /tmp/venv

RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH=$HOME/.local/bin:$PATH


ADD . /repo
WORKDIR /repo


# ---------------------------------------------
# STAGE: rust-builder
# --------------------------------------------
# Installs rustc and cargo using rust-up
FROM ubuntu:22.04 AS rust-base
RUN apt update && apt install curl build-essential -qqy

ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo

ENV RUST_VERSION=1.74
RUN curl \
    --proto '=https' \
    --tlsv1.2 \
    -sSf https://sh.rustup.rs | sh \
    -s -- -y --default-toolchain ${RUST_VERSION}

ENV PATH=$CARGO_HOME/bin:$PATH

RUN rustup default stable

# Ensure we can use cargo under a different user
RUN chmod a+rwx -R $CARGO_HOME

# -------------------------------------
# STAGE: bitcoin-downloader
# ------------------------------------
# Downloads bitcoin-core
FROM ubuntu:22.04 AS bitcoin-downloader
RUN apt update && apt install -qqy wget
ARG TARGETPLATFORM

ARG BITCOIN_VERSION=25.0

RUN PLATFORM=$( \
    case ${TARGETPLATFORM} in \
    linux/amd64 ) echo "x86_64";; \
    linux/arm64 ) echo "aarch64";; \
    esac \
    ) && \
    cd /tmp/ && \
    wget "https://bitcoincore.org/bin/bitcoin-core-${BITCOIN_VERSION}/bitcoin-${BITCOIN_VERSION}-${PLATFORM}-linux-gnu.tar.gz" -O bitcoin.tar.gz && \
    tar -xvzf bitcoin.tar.gz && \
    mv /tmp/bitcoin-$BITCOIN_VERSION/ /opt/bitcoin && \
    rm -rf bitcoin.tar.gz /tmp/bitcoin-$BITCOIN_VERSION

# ---------------------------------------
# STAGE: cfssl-downloader
# --------------------------------------
# Downloads cloudlfares SSL binaries
FROM ubuntu:22.04 AS cfssl-downloader
ARG TARGETARCH

RUN apt update && apt install -qqy wget

RUN wget -q https://github.com/cloudflare/cfssl/releases/download/v1.6.5/cfssl_1.6.5_linux_${TARGETARCH} -O /usr/bin/cfssl && \
    chmod a+x /usr/bin/cfssl
RUN wget -q https://github.com/cloudflare/cfssl/releases/download/v1.6.5/cfssljson_1.6.5_linux_${TARGETARCH} -O /usr/bin/cfssljson && \
    chmod a+x /usr/bin/cfssljson

# ----------------------------------------
# STAGE: protoc-downloader
# ----------------------------------------
FROM ubuntu:22.04 AS protoc-downloader
ARG TARGETPLATFORM

RUN apt update && apt install -qqy wget unzip
RUN PLATFORM=$( \
    case ${TARGETPLATFORM} in \
    linux/amd64 ) echo "x86_64";; \
    linux/arm64 ) echo "aarch_64";; \
    esac \
    ) && \
    mkdir /tmp/protoc && \
    cd /tmp/protoc && \
    wget --quiet -O protoc.zip https://github.com/protocolbuffers/protobuf/releases/download/v3.19.3/protoc-3.19.3-linux-${PLATFORM}.zip && \
    unzip protoc.zip && \
    mv /tmp/protoc/bin/protoc /usr/local/bin && \
    chmod a+x /usr/local/bin/protoc && \
    rm -rf /tmp/protoc

# ----------------------------------------
# STAGE: plugin-builder
# ---------------------------------------
# Create a release installation or all plugins.
# This is mainly useful for users who don't to build or edit the rust-code
FROM rust-base AS gl-plugin-builder

RUN apt update && apt install -qqy git

# Building the grpc-interface and gl-plugin requires a protoc compiler
COPY --from=protoc-downloader /usr/local/bin/protoc /usr/local/bin/protoc

# We should only ADD the files we need to optimize caching
ADD . /repo

WORKDIR /repo

RUN cd /repo/libs/gl-plugin && cargo build && \
    cd /repo/libs/gl-signerproxy && cargo build

# ---------------------
# STAGE: Dev container
# ----------------------------
#
# This is the dev-container that we will be using
FROM rust-base AS gl-dev
ENV DEBIAN_FRONTEND=noninteractive
# Indicate we run inside the Dockerfile
ENV GL_DOCKER=1
# Tells cln to use shorter polling intervals
ENV DEVELOPER=1
ENV PYTHONUNBUFFERED=1
# Start by setting the PATH once and for all, they might not be there
# yet, but we will put the tools in those directories.
ENV PATH=$HOME/.local/bin:/opt/cln-latest/usr/local/bin:/opt/bitcoin/bin:/opt/cln-latest/usr/local/bin:/usr/local/bin:$PATH
ENV UV_INSTALL_DIR=/usr/local/bin/
# grpcio == 1.46 produces spammy log messages, silence them
ENV GRPC_ENABLE_FORK_SUPPORT=0

# Install cln-versions
ENV CLNVM_CACHE_DIR=/opt/cln

# Enumerate all versions that gl-testing should find
ENV CLN_PATH=/opt/cln/v0.10.1/usr/local/bin/:/opt/cln/v0.10.2/usr/local/bin/:/opt/cln/v0.11.0.1/usr/local/bin/:/opt/cln/v0.11.2gl2/usr/local/bin/:/opt/cln/v22.11gl1/usr/local/bin/:/opt/cln/v23.05gl1/usr/local/bin/:/opt/cln/v23.08gl1/usr/local/bin/:/opt/cln/v24.02gl1/usr/local/bin/:/opt/cln/v24.11gl1/usr/local/bin/

# Create a non-root user.
ARG GID=0
ARG UID=0
ARG DOCKER_USER=dev
ENV UV_LINK_MODE=copy
# Configure rust and related logging. Set up a default logging
# filter. This includes all trace-level messages from greenlight
# components and validating lightning signer
ENV RUST_LOG=gl_client=trace,tracing=warn,gl_signerproxy=trace,gl_plugin=trace,lightning_signer=trace,vls_protocol_signer=trace,vls_core=trace,vls_persist=trace,vls_protocol=trace,info

# Required dependencies and some dependencies which are nice to have
# floating around
RUN apt-get update && apt install -qqy \
    curl \
    git \
    jq \
    libpq5 \
    python3 \
    python3-venv \
    python3-pip \
    tree \
    socat \
    sudo \
    unzip \
    wget \
    && rm -rf /var/lib/apt/lists/*


# Install bitcoin-core
COPY --from=bitcoin-downloader /opt/bitcoin/bin /opt/bitcoin/bin

# Install cfssl
COPY --from=cfssl-downloader /usr/bin/cfssl /usr/local/bin/cfssl
COPY --from=cfssl-downloader /usr/bin/cfssljson /usr/local/bin/cfssljson

# Install protoc
COPY --from=protoc-downloader /usr/local/bin/protoc /usr/local/bin/protoc

# Check if the group already exists before running groupadd
RUN getent group $GID || groupadd -g $GID -o $DOCKER_USER && \
    id -u $UID || useradd -m -u $UID -g $GID -G sudo -o -s /bin/bash $DOCKER_USER && \
    echo '%sudo ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

RUN mkdir -p /tmp/gltesting/cargo && mkdir -p /tmp/gltesting/tmp
RUN sh -c "$(curl --location https://taskfile.dev/install.sh)" -- -d
RUN curl -LsSf https://astral.sh/uv/install.sh | sh

ADD . /repo
WORKDIR /repo

# Populate `uv` cache
RUN uv sync --all-packages --dev

# Populate CLN versions
RUN uv run python3 -m clnvm --verbose get-all

# Create a symlink to the latest cln-version and add it to the path
RUN ln -s $(uv run --package cln-version-manager latest --root-path) /opt/cln-latest
