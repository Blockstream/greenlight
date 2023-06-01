FROM ubuntu:20.04
ENV TZ="Europe/Zurich"
ENV DEBIAN_FRONTEND=noninteractive
ENV RUST_VERSION=1.67.1
ENV PATH=$PATH:/root/.cargo/bin:/root/.local/bin:/root/.pyenv/bin:/root/.pyenv/shims
ARG DOCKER_USER=default_user
RUN adduser $DOCKER_USER

ENV CARGO_TARGET_DIR=/tmp/cargo-target
RUN apt-get update -qq && \
    apt-get install -y \
    autoconf \
    clang \
    curl \
    git \
    jq \
    libgmp-dev \
    libsqlite3-dev \
    libtool \
    make \
    pkg-config \
    python3-pip \
    protobuf-compiler \
    unzip \
    wget \
    zlib1g-dev

USER $DOCKER_USER
ENV HOME=/home/${DOCKER_USER}
ENV PATH=$PATH:$HOME/.cargo/bin:$HOME/.local/bin:$HOME/.pyenv/bin:$HOME/.pyenv/shims:$HOME/.nodenv/bin:$HOME/.nodenv/shims

RUN git clone https://github.com/nodenv/nodenv.git ~/.nodenv && \
    mkdir -p "$(nodenv root)"/plugins && \
    git clone https://github.com/nodenv/node-build.git "$(nodenv root)"/plugins/node-build && \
    nodenv install 20.0.0 && nodenv global 20.0.0

RUN npm install -g cargo-cp-artifact

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION} && \
    ~/.cargo/bin/rustup toolchain install ${RUST_VERSION} --component rustfmt --allow-downgrade
RUN git config --global --add safe.directory /io/public
RUN git config --global --add safe.directory /io
WORKDIR /io
