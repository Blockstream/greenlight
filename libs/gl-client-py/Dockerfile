# Dockerfile used to build manylinux packages
FROM ubuntu:18.04

ENV TZ="Europe/Zurich"
ENV DEBIAN_FRONTEND=noninteractive
ENV RUST_VERSION=1.67.1
ENV PATH=$PATH:/root/.cargo/bin:/root/.local/bin:/root/.pyenv/bin:/root/.pyenv/shims
# Min version PyO3 supports
ENV PYO3_PYTHON=python3.7

# Set the Cargo target directory, so we don't end up polluting the
# host's repo with our own artifacts. Wheels should be copied to
# `libs/wheelhouse`, so no problem doing this here.
ENV CARGO_TARGET_DIR=/tmp/cargo-target
RUN apt-get update -qq && \
    apt-get install -y \
    autoconf \
    clang \
    curl \
    git \
    libgmp-dev \
    libsqlite3-dev \
    libtool \
    make \
    pkg-config \
    python3-pip \
    unzip \
    wget \
    zlib1g-dev

# Pyenv dependencies
RUN apt-get install -y \
    libbz2-dev \
    libreadline-dev \
    libedit-dev \
    libssl-dev\
    libncursesw5-dev \
    xz-utils \
    tk-dev \
    libxml2-dev \
    libxmlsec1-dev \
    libffi-dev \
    liblzma-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION} && \
    ~/.cargo/bin/rustup toolchain install ${RUST_VERSION} --component rustfmt --allow-downgrade

RUN mkdir /tmp/protoc && cd /tmp/protoc && \
    wget --quiet -O protoc.zip https://github.com/protocolbuffers/protobuf/releases/download/v3.19.3/protoc-3.19.3-linux-x86_64.zip && \
    unzip protoc.zip && \
    mv /tmp/protoc/bin/protoc /usr/local/bin && \
    rm -rf /tmp/protoc

RUN  git clone https://github.com/pyenv/pyenv.git ~/.pyenv && \
    pyenv install 3.7.12 && \
    pyenv install 3.8.12 && \
    pyenv install 3.9.9 && \
    pyenv install 3.10.1 && \
    pyenv global 3.7.12 3.8.12 3.9.9 3.10.1

RUN python3 -m pip install -U pip tomli && \
    python3 -m pip install maturin mako mrkd cffi

WORKDIR /io/
CMD bash -c "cd /io/libs/gl-client-py && \
    cargo build --release && \
    maturin build --release --out=/io/wheelhouse --compatibility=2_27"
