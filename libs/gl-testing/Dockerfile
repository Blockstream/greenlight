# A simple Dockerfile that can be used to test integrations.
#
# The `gl-testing` framework requires a number of dependencies in
# order to run correctly (CLN versions, python dependencies,
# `gl-client` library, etc). This Dockerfile provisions all those
# dependencies correctly and can be used to test the greenlight
# libraries themselves or as a base image for your own test images.

FROM ubuntu:20.04 as builder
ENV TZ="Europe/Zurich"
ENV DEBIAN_FRONTEND=noninteractive
ENV RUST_VERSION=1.70.0
ENV PATH=/root/.cargo/bin:$PATH
ENV RUST_LOG=debug
ENV PYTHONPATH=/repo/libs/gl-testing
ENV CARGO_UNSTABLE_SPARSE_REGISTRY=true

RUN apt-get update && apt-get install -qqy \
    python3 \
    python3-pip \
    clang \
    autoconf \
    automake \
    build-essential \
    git \
    jq \
    libtool \
    libpq5 \
    libgmp-dev \
    libsqlite3-dev \
    net-tools \
    zlib1g-dev \
    libsodium-dev \
    gettext \
    pkgconf \
    curl \
    wget \
    unzip \
    && rm -rf /var/lib/apt/lists/*

RUN curl \
    --proto '=https' \
    --tlsv1.2 \
    -sSf https://sh.rustup.rs | sh \
    -s -- -y --default-toolchain ${RUST_VERSION}

RUN mkdir /tmp/protoc && \
    cd /tmp/protoc && \
    wget --quiet -O protoc.zip https://github.com/protocolbuffers/protobuf/releases/download/v3.19.3/protoc-3.19.3-linux-x86_64.zip && \
    unzip protoc.zip && \
    mv /tmp/protoc/bin/protoc /usr/local/bin && \
    rm -rf /tmp/protoc

RUN python3 -m pip install -U \
      pip tomli &&\
      python3 -m pip install \
      maturin[patchelf] \
      mrkd \
      mako
ADD . /repo
WORKDIR /repo

RUN make cln

RUN cd /repo/libs/gl-plugin && cargo build --release && \
    cd /repo/libs/gl-signerproxy && cargo build --release

# Move the binaries into the location the tester expects, we copy just
# the binaries and we switch from release to debug, simply to keep the
# docker image size in check
RUN mkdir -p /tmp/target/debug && \
    mv /repo/libs/target/release/gl-plugin /tmp/target/debug/ && \
    mv /repo/libs/target/release/gl-signerproxy /tmp/target/debug/

FROM ubuntu:20.04 as tester
ARG BITCOIN_VERSION=25.0
ARG DOCKER_USER=default_user
ARG UID=1000
ARG GID=1000
ENV TZ="Europe/Zurich"
ENV DEBIAN_FRONTEND=noninteractive
ENV CARGO_TARGET_DIR=/tmp/target/
ENV RUST_VERSION=1.70.0

RUN groupadd -g $GID -o $DOCKER_USER &&\
    useradd -m -u $UID -g $GID -o -s /bin/bash $DOCKER_USER

# Set up a default logging filter. This includes all trace-level messages from greenlight components and validating lightning signer
ENV RUST_LOG=gl_client=trace,tracing=warn,gl_signerproxy=trace,gl_plugin=trace,lightning_signer=trace,vls_protocol_signer=trace,vls_core=trace,vls_persist=trace,vls_protocol=trace,info

# grpcio == 1.46 produces spammy log messages, silence them
ENV GRPC_ENABLE_FORK_SUPPORT=0

RUN apt-get update && apt-get install -qqy \
    autoconf \
    clang \
    curl \
    git \
    libgmp-dev \
    libsodium23 \
    libsqlite3-dev \
    libpq5 \
    libtool \
    nodejs \
    npm \
    pkgconf \
    python3 \
    python3-pip \
    python3-venv \
    nodejs \
    npm \
    socat \
    unzip \
    wget \
    jq \
    && rm -rf /var/lib/apt/lists/*

# Enumerate all CLN version we want `gl-testing` to find.
ENV CLN_PATH=/opt/cln/v0.10.1/usr/local/bin/:/opt/cln/v0.10.2/usr/local/bin/:/opt/cln/v0.11.0.1/usr/local/bin/:/opt/cln/v0.11.2gl2/usr/local/bin/:/opt/cln/v22.11gl1/usr/local/bin/:/opt/cln/v23.05gl1/usr/local/bin/:/opt/cln/v23.08gl1/usr/local/bin/

# Shorten poll intervals so tests are faster to run
ENV DEVELOPER=1

# Add the latest CLN version to the `$PATH` so `pyln-testing` can find
# it when it wants to spawn a new node
ENV PATH=/opt/cln-latest/usr/local/bin/:$PATH

# And we also add the cargo bin directory just for the sake of
# simplifying a dev's life.
ENV PATH=/home/$DOCKER_USER/.cargo/bin:$PATH

# Make sure we can tell when running in the docker container. This is
# helpful if we expect to be run in the container, but have been
# invoked somewhere else.
ENV GL_DOCKER=1
ENV PATH=/tmp/venv/bin:$PATH
ENV VIRTUAL_ENV=/tmp/venv
ENV NPM_PACKAGES="${HOME}/.npm-packages"
ENV PATH="${PATH}:${NPM_PACKAGES}/bin"

RUN cd /tmp/ && \
    wget "https://bitcoincore.org/bin/bitcoin-core-${BITCOIN_VERSION}/bitcoin-${BITCOIN_VERSION}-x86_64-linux-gnu.tar.gz" -O bitcoin.tar.gz && \
    tar -xvzf bitcoin.tar.gz && \
    mv /tmp/bitcoin-$BITCOIN_VERSION/bin/bitcoin* /usr/local/bin/ && \
    rm -rf bitcoin.tar.gz /tmp/bitcoin-$BITCOIN_VERSION

COPY --from=builder /repo/cln-versions/ /opt/cln
RUN ln -s /opt/cln/v23.05gl1 /opt/cln-latest

RUN mkdir /tmp/protoc && \
    cd /tmp/protoc && \
    wget --quiet -O protoc.zip https://github.com/protocolbuffers/protobuf/releases/download/v3.19.3/protoc-3.19.3-linux-x86_64.zip && \
    unzip protoc.zip && \
    mv /tmp/protoc/bin/* /usr/local/bin/ && \
    rm -rf protoc.zip /tmp/protoc && \
    chmod a+x /usr/local/bin/protoc

RUN wget -q https://github.com/cloudflare/cfssl/releases/download/v1.5.0/cfssl_1.5.0_linux_amd64 -O /usr/bin/cfssl && \
    chmod a+x /usr/bin/cfssl
RUN wget -q https://github.com/cloudflare/cfssl/releases/download/v1.5.0/cfssljson_1.5.0_linux_amd64 -O /usr/bin/cfssljson && \
    chmod a+x /usr/bin/cfssljson

COPY --from=builder /repo/cln-versions/ /opt/cln
COPY --from=builder /tmp/target /tmp/target
ADD . /repo
RUN chown $DOCKER_USER -R /tmp/target /repo

# Switch to docker user bevore we install dependencies.
USER $DOCKER_USER

# Add cargo and the rust toolchain in case we want to compile
# gl-plugin and gl-signerproxy. This can be useful if we want to
# develop them, and bind-mount the repository from the host into
# `/repo`
RUN curl \
    --proto '=https' \
    --tlsv1.2 \
    -sSf https://sh.rustup.rs | sh \
    -s -- -y --default-toolchain ${RUST_VERSION}

# Run in a venv, so we don't get warnings about root being special.
RUN python3 -m venv /tmp/venv

RUN python3 -m pip install -U pip tomli && \
    python3 -m pip install \
    mako \
    protobuf \
    ephemeral-port-reserve \
    sh \
    pytest-timeout \
    pytest-xdist \
    rich \
    pyln-testing \
    pytest-cov \
    grpcio \
    pytest-xdist \
    maturin[patchelf] \
    grpcio-tools \
    purerpc[grpc] \
    mypy-protobuf

RUN git config --global --add safe.directory /repo

# Install nodejs
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.1/install.sh | bash
RUN mkdir "${HOME}/.npm-packages" && npm config set prefix "${HOME}/.npm-packages"

# Required to build and test the JS bindings
RUN npm install -g cargo-cp-artifact

RUN (cd /repo/libs/gl-client-py; maturin develop)
# Install `gl-testing`. When integrators use this image they'll mount
# their own code into `/repo`
RUN python3 -m pip install /repo/libs/gl-testing

# Hack: importing the proto files twice isn't a good idea, and
# pyln-testing currently does.
RUN sed -i \
    's/from pyln.testing import grpc/#from pyln.testing import grpc/g' \
    /tmp/venv/lib/python3.8/site-packages/pyln/testing/utils.py

WORKDIR /repo
