# Dockerfile used to build manylinux packages
FROM ubuntu:18.04

ENV TZ="Europe/Zurich"
ENV DEBIAN_FRONTEND=noninteractive
ENV RUST_VERSION=nightly-2022-01-14
ENV PATH=$PATH:/root/.cargo/bin:/root/.local/bin:/root/.pyenv/bin:/root/.pyenv/shims

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
    pyenv install 3.6.15 && \
    pyenv install 3.7.12 && \
    pyenv install 3.8.12 && \
    pyenv install 3.9.9 && \
    pyenv install 3.10.1 && \
    pyenv global 3.6.15 3.7.12 3.8.12 3.9.9 3.10.1

RUN python3 -m pip install -U pip tomli && \
    python3 -m pip install maturin mako mrkd cffi

WORKDIR /io/gl-client-py
CMD bash -c "cargo build --release && \
    maturin build --release --out=../dist --compatibility=2_27"
