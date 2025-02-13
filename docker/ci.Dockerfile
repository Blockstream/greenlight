FROM python:3.9-bullseye

SHELL ["/bin/bash", "-c"]

RUN pip install --upgrade pip \
  && pip install --upgrade maturin wheel poetry

ENV RUST_VERSION=1.83
# ENV RUST_DIST_VERSION=v0.13.3
ENV RUST_DIST_VERSION=v0.4.2

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- --default-toolchain ${RUST_VERSION} -y \
  && curl --proto '=https' --tlsv1.2 -LsSf https://github.com/axodotdev/cargo-dist/releases/download/${RUST_DIST_VERSION}/cargo-dist-installer.sh | bash

RUN mkdir /tmp/protoc && cd /tmp/protoc && \
    wget --quiet -O protoc.zip https://github.com/protocolbuffers/protobuf/releases/download/v25.2/protoc-25.2-linux-x86_64.zip && \
    unzip protoc.zip && \
    mv /tmp/protoc/bin/protoc /usr/local/bin && \
    chmod a+x /usr/local/bin/protoc && \
    rm -rf /tmp/protoc
