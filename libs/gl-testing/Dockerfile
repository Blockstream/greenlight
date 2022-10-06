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
ENV RUST_VERSION=1.63.0
ENV PATH=/root/.cargo/bin:$PATH

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
      pip \
    maturin \
    mrkd \
    mako

ENV VERSION=v0.10.1
RUN git clone https://github.com/ElementsProject/lightning.git /repo/cln-$VERSION --branch $VERSION && \
    cd /repo/cln-$VERSION && \
    pip install -r requirements.txt && \
    ./configure --enable-developer --disable-valgrind && \
    make -j $(nproc) && \
    make DESTDIR=/opt/cln-$VERSION   install

ENV VERSION=v0.10.2
RUN git clone https://github.com/ElementsProject/lightning.git /repo/cln-$VERSION --branch $VERSION && \
    cd /repo/cln-$VERSION && \
    pip install -r requirements.lock && \
    ./configure --enable-developer --disable-valgrind && \
    make -j $(nproc) && \
    make DESTDIR=/opt/cln-$VERSION   install

ENV VERSION=v0.11.0.1
RUN git clone https://github.com/ElementsProject/lightning.git /repo/cln-$VERSION --branch $VERSION && \
    cd /repo/cln-$VERSION && \
    ./configure --enable-developer --disable-valgrind && \
    make -j $(nproc) && \
    make DESTDIR=/opt/cln-$VERSION   install

ENV VERSION=v0.11.2gl2
RUN git clone https://github.com/cdecker/lightning.git /repo/cln-$VERSION --branch $VERSION && \
    cd /repo/cln-$VERSION && \
    ./configure --enable-developer --disable-valgrind && \
    make -j $(nproc) && \
    make DESTDIR=/opt/cln-$VERSION   install

# Place a link to the latest version so we can add it to the path in
# the tester image.
RUN ln -s /opt/cln-$VERSION /opt/cln-latest

ADD . /repo
ENV RUST_LOG=debug
RUN (cd /repo/libs/gl-client && cargo build --release) && \
    (cd /repo/libs/gl-client-py && maturin build --release --strip --compatibility manylinux_2_28) && \
    (cd /repo/libs && cargo build --all --release --bin gl-plugin && cargo build --all --release --bin gl-signerproxy)

# Move the binaries into the location the tester expects, we copy just
# the binaries and we switch from release to debug, simply to keep the
# docker image size in check
RUN mkdir -p /tmp/target/debug && \
    mv /repo/libs/target/release/gl-plugin /tmp/target/debug/ && \
    mv /repo/libs/target/release/gl-signerproxy /tmp/target/debug/

FROM ubuntu:20.04 as tester
ARG BITCOIN_VERSION=0.20.1
ENV TZ="Europe/Zurich"
ENV DEBIAN_FRONTEND=noninteractive
ENV CARGO_TARGET_DIR=/tmp/target/
ENV RUST_VERSION=1.63.0

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
    libtool \
    nodejs \
    npm \
    pkgconf \
    python3 \
    python3-pip \
    python3-venv \
    socat \
    unzip \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Enumerate all CLN version we want `gl-testing` to find.
ENV CLN_PATH=/opt/cln-v0.10.1/usr/local/bin/:/opt/cln-v0.10.2/usr/local/bin/:/opt/cln-v0.11.0.1/usr/local/bin/:/opt/cln-v0.11.2gl2/usr/local/bin/

# Shorten poll intervals so tests are faster to run
ENV DEVELOPER=1

# Add the latest CLN version to the `$PATH` so `pyln-testing` can find
# it when it wants to spawn a new node
ENV PATH=/opt/cln-latest/usr/local/bin/:$PATH

# And we also add the cargo bin directory just for the sake of
# simplifying a dev's life.
ENV PATH=/root/.cargo/bin:$PATH

# Make sure we can tell when running in the docker container. This is
# helpful if we expect to be run in the container, but have been
# invoked somewhere else.
ENV GL_DOCKER=1

# Add cargo and the rust toolchain in case we want to compile
# gl-plugin and gl-signerproxy. This can be useful if we want to
# develop them, and bind-mount the repository from the host into
# `/repo`
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

RUN wget -q https://github.com/cloudflare/cfssl/releases/download/v1.5.0/cfssl_1.5.0_linux_amd64 -O /usr/bin/cfssl && \
    chmod a+x /usr/bin/cfssl
RUN wget -q https://github.com/cloudflare/cfssl/releases/download/v1.5.0/cfssljson_1.5.0_linux_amd64 -O /usr/bin/cfssljson && \
    chmod a+x /usr/bin/cfssljson

# Run in a venv, so we don't get warnings about root being special.
RUN python3 -m venv /tmp/venv
ENV PATH=/tmp/venv/bin:$PATH

RUN python3 -m pip install -U pip && \
    python3 -m pip install \
    mako \
    protobuf \
    ephemeral-port-reserve \
    sh \
    pytest-timeout \
    pytest-xdist \
    rich \
    pyln-testing \
    grpcio \
    pytest-xdist \
    maturin

# Install nodejs
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.1/install.sh | bash

RUN cd /tmp/ && \
    wget https://storage.googleapis.com/c-lightning-tests/bitcoin-$BITCOIN_VERSION-x86_64-linux-gnu.tar.bz2 -O bitcoin.tar.bz2 && \
    tar -xvjf bitcoin.tar.bz2 && \
    mv /tmp/bitcoin-$BITCOIN_VERSION/bin/bitcoin* /usr/local/bin/ && \
    rm -rf bitcoin.tar.bz2 /tmp/bitcoin-$BITCOIN_VERSION

COPY --from=builder /opt/ /opt/
COPY --from=builder /tmp/target /tmp/target
COPY --from=builder /repo/libs/target/wheels/gl_client_py-* /opt/
RUN python3 -m pip install /opt/gl_client_py-*
ADD . /repo

# Install `gl-testing`. When integrators use this image they'll mount
# their own code into `/repo`
RUN python3 -m pip install /repo/libs/gl-testing

WORKDIR /repo
