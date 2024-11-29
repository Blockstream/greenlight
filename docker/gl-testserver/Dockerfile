FROM ubuntu:22.04 AS python-builder

ENV RUST_VERSION=1.74
ENV PATH=$CARGO_HOME/bin:$PATH
ENV PROTOC_VERSION=3.19.3
ENV CFSSL_VERSION=1.6.5
ENV GL_TESTING_IGNORE_HASH=true
ARG BITCOIN_VERSION=24.0
ARG GID=0
ARG UID=0
ARG DOCKER_USER=dev
ENV PATH=$PATH:/home/$DOCKER_USER/.local/bin/:/opt/bitcoin/bin:/home/$DOCKER_USER/.cargo/bin
#ENV VIRTUAL_ENV=/tmp/venv
ENV CARGO_TARGET_DIR=/tmp/cargo

RUN apt update && apt install -qqy \
    curl \
    python3 \
    python3-pip \
    python3-venv \
    libpq-dev \
    unzip \
    sudo \
    git \
    build-essential \
    wget

RUN groupadd -g $GID -o $DOCKER_USER &&\
    useradd -m -u $UID -g $GID -G sudo -o -s /bin/bash $DOCKER_USER && \
    echo '%sudo   ALL=(ALL:ALL) ALL' >> /etc/sudoers

RUN wget -q https://github.com/cloudflare/cfssl/releases/download/v${CFSSL_VERSION}/cfssl_${CFSSL_VERSION}_linux_amd64 -O /usr/bin/cfssl && \
    chmod a+x /usr/bin/cfssl
RUN wget -q https://github.com/cloudflare/cfssl/releases/download/v${CFSSL_VERSION}/cfssljson_${CFSSL_VERSION}_linux_amd64 -O /usr/bin/cfssljson && \
    chmod a+x /usr/bin/cfssljson

RUN mkdir /tmp/protoc && \
    cd /tmp/protoc && \
    wget --quiet \
      -O protoc.zip \
      https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-x86_64.zip && \
    unzip protoc.zip && \
    sudo mv /tmp/protoc/bin/protoc /usr/local/bin && \
    chmod a+x /usr/local/bin/protoc && \
    rm -rf /tmp/protoc

RUN cd /tmp/ && \
    wget "https://bitcoincore.org/bin/bitcoin-core-${BITCOIN_VERSION}/bitcoin-${BITCOIN_VERSION}-x86_64-linux-gnu.tar.gz" -O bitcoin.tar.gz && \
    tar -xvzf bitcoin.tar.gz && \
    mv /tmp/bitcoin-$BITCOIN_VERSION/ /opt/bitcoin && \
    rm -rf bitcoin.tar.gz /tmp/bitcoin-$BITCOIN_VERSION

ADD ../ /repo/libs
RUN chown $DOCKER_USER:users -R /repo

USER $DOCKER_USER

RUN curl \
    --proto '=https' \
    --tlsv1.2 \
    -sSf https://sh.rustup.rs | sh \
    -s -- -y --default-toolchain ${RUST_VERSION}
RUN rustup default stable

RUN curl -LsSf https://astral.sh/uv/install.sh | sh

WORKDIR /repo/libs/gl-testserver/
RUN echo $PATH
RUN uv sync --locked -v
RUN uv run clnvm get-all
CMD uv run gltestserver run --metadata /tmp/gltestserver
