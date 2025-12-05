############################
# Stage 1: builder
############################
FROM rust:slim-bookworm AS builder

#---------------------------------------------------------------------
# Build-time arguments & environment variables
#---------------------------------------------------------------------
ARG BITCOIN_VERSION=28.2
ARG GID=0
ARG UID=0
ARG DOCKER_USER=dev
ARG REPO_PATH=/repo
ARG TARGETPLATFORM
ARG TARGETARCH

ENV UV_PYTHON=3.10
ENV PROTOC_VERSION=3.19.3
ENV CFSSL_VERSION=1.6.5
ENV PATH=/home/${DOCKER_USER}/.local/bin:/opt/bitcoin/bin:/usr/local/bin:$PATH
ENV CARGO_TARGET_DIR=/tmp/target

#---------------------------------------------------------------------
# Base packages required for building
#---------------------------------------------------------------------
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive \
    apt-get install -y --no-install-recommends \
    build-essential \
    git curl wget unzip sudo \
    && rm -rf /var/lib/apt/lists/*

#---------------------------------------------------------------------
# Additional Rust components
#---------------------------------------------------------------------
RUN rustup component add rustfmt

#---------------------------------------------------------------------
# Create dev user with password-less sudo
#---------------------------------------------------------------------
RUN groupadd -g ${GID} -o ${DOCKER_USER} && \
    useradd  -m -u ${UID} -g ${GID} -G sudo -o -s /bin/bash ${DOCKER_USER} && \
    echo '%sudo ALL=(ALL:ALL) NOPASSWD:ALL' >> /etc/sudoers

USER ${DOCKER_USER}
WORKDIR /home/${DOCKER_USER}

#---------------------------------------------------------------------
# Directories for build tools and staged artifacts
#---------------------------------------------------------------------
RUN sudo mkdir -p /opt/build-tools /opt/bitcoin /opt/stage && \
    sudo chown ${DOCKER_USER}:${DOCKER_USER} /opt/build-tools /opt/bitcoin /opt/stage

#---------------------------------------------------------------------
# cfssl & cfssljson
#---------------------------------------------------------------------
RUN wget -q https://github.com/cloudflare/cfssl/releases/download/v${CFSSL_VERSION}/cfssl_${CFSSL_VERSION}_linux_${TARGETARCH} -O /opt/build-tools/cfssl && \
    wget -q https://github.com/cloudflare/cfssl/releases/download/v${CFSSL_VERSION}/cfssljson_${CFSSL_VERSION}_linux_${TARGETARCH} -O /opt/build-tools/cfssljson && \
    chmod +x /opt/build-tools/cfssl /opt/build-tools/cfssljson

#---------------------------------------------------------------------
# protoc
#---------------------------------------------------------------------
RUN PLATFORM=$(case "${TARGETPLATFORM}" in \
    linux/amd64) echo "x86_64";; \
    linux/arm64) echo "aarch_64";; \
    esac) && \
    wget -q https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-${PLATFORM}.zip -O /tmp/protoc.zip && \
    unzip -q /tmp/protoc.zip -d /tmp/protoc && \
    mv /tmp/protoc/bin/protoc /opt/build-tools/protoc && \
    chmod +x /opt/build-tools/protoc && \
    rm -rf /tmp/protoc /tmp/protoc.zip

#---------------------------------------------------------------------
# bitcoind (static binaries)
#---------------------------------------------------------------------
RUN ARCH=$(case "${TARGETPLATFORM}" in \
    linux/amd64) echo "x86_64";; \
    linux/arm64) echo "aarch64";; \
    esac) && \
    wget -q https://bitcoincore.org/bin/bitcoin-core-${BITCOIN_VERSION}/bitcoin-${BITCOIN_VERSION}-${ARCH}-linux-gnu.tar.gz -O /tmp/bitcoin.tgz && \
    tar -xf /tmp/bitcoin.tgz -C /tmp && \
    mv /tmp/bitcoin-${BITCOIN_VERSION}/* /opt/bitcoin && \
    rm -rf /tmp/bitcoin.tgz /tmp/bitcoin-${BITCOIN_VERSION}

ENV PATH=/opt/build-tools:$PATH

#---------------------------------------------------------------------
# Copy source code
#---------------------------------------------------------------------
WORKDIR ${REPO_PATH}
COPY --chown=${DOCKER_USER}:users ../../Cargo.toml ${REPO_PATH}/
COPY --chown=${DOCKER_USER}:users ../../pyproject.toml ${REPO_PATH}/
COPY --chown=${DOCKER_USER}:users ../../libs ${REPO_PATH}/libs
COPY --chown=${DOCKER_USER}:users ../../examples ${REPO_PATH}/examples

#---------------------------------------------------------------------
# Build Rust binaries (release, stripped)
#---------------------------------------------------------------------
WORKDIR ${REPO_PATH}
RUN cargo build --release \
    --manifest-path libs/gl-plugin/Cargo.toml --bin gl-plugin && \
    cargo build --release \
    --manifest-path libs/gl-signerproxy/Cargo.toml --bin gl-signerproxy && \
    strip ${CARGO_TARGET_DIR}/release/gl-plugin ${CARGO_TARGET_DIR}/release/gl-signerproxy

# Build gl-sdk and copy the shared library for Python bindings
RUN cargo build -p gl-sdk && \
    cp ${CARGO_TARGET_DIR}/debug/libglsdk.so libs/gl-sdk/glsdk/

#---------------------------------------------------------------------
# Lock and sync Python dependencies using uv
#---------------------------------------------------------------------
WORKDIR ${REPO_PATH}/libs/gl-testserver
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /bin/
RUN --mount=type=cache,target=/root/.cache/uv \
    uv lock && uv sync --locked -v --no-editable --all-packages --dev

#---------------------------------------------------------------------
# Collect all artifacts into /opt/stage
#---------------------------------------------------------------------
RUN mkdir -p /opt/stage/bin /opt/stage/bitcoin && \
    # Rust binaries
    cp ${CARGO_TARGET_DIR}/release/gl-plugin ${CARGO_TARGET_DIR}/release/gl-signerproxy /opt/stage/bin/ && \
    # Helper tools
    cp /opt/build-tools/* /opt/stage/bin/ && \
    # bitcoind
    cp -r /opt/bitcoin /opt/stage/ && \
    # uv binaries
    cp /bin/uv /bin/uvx /opt/stage/bin/ && \
    # uv virtual environment
    cp -r ${REPO_PATH}/.venv /opt/stage/uv

############################
# Stage 2: runtime
############################
FROM ubuntu:24.04 AS final

#---------------------------------------------------------------------
# Minimal runtime packages
#---------------------------------------------------------------------
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive \
    apt-get install -y --no-install-recommends \
    libpq5 curl openssl ca-certificates build-essential git gnupg libsodium23 \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

#---------------------------------------------------------------------
# Arguments & environment variables (kept identical with builder)
#---------------------------------------------------------------------
ARG GID=0
ARG UID=0
ARG DOCKER_USER=dev
ARG REPO_PATH=/repo

ENV UV_PYTHON=3.10
ENV PATH=/home/${DOCKER_USER}/.local/bin:/usr/local/bin:/opt/bitcoin/bin:$PATH
ENV REPO_PATH=${REPO_PATH}
ENV GL_TESTING_IGNORE_HASH=False

#---------------------------------------------------------------------
# Create runtime user
#---------------------------------------------------------------------
RUN if getent passwd $UID > /dev/null; then \
        existing_user=$(getent passwd $UID | cut -d: -f1); \
        if [ "$existing_user" != "$DOCKER_USER" ]; then \
            usermod -l $DOCKER_USER -d /home/$DOCKER_USER -m $existing_user; \
            groupmod -n $DOCKER_USER $(getent group $GID | cut -d: -f1); \
        fi; \
    else \
        groupadd --gid $GID $DOCKER_USER; \
        useradd --uid $UID --gid $GID -m $DOCKER_USER; \
    fi

USER ${DOCKER_USER}
#---------------------------------------------------------------------
# Copy artifacts from the builder stage
#---------------------------------------------------------------------
COPY --from=builder /opt/stage/bin/*      /usr/local/bin/
COPY --from=builder /opt/stage/bitcoin    /opt/bitcoin

#---------------------------------------------------------------------
# Application code (copy only libs to keep image small)
#---------------------------------------------------------------------
WORKDIR ${REPO_PATH}
COPY --chown=${DOCKER_USER}:${DOCKER_USER} . ${REPO_PATH}
COPY --from=builder /opt/stage/uv ${REPO_PATH}/.venv

#---------------------------------------------------------------------
# ENTRYPOINT
#---------------------------------------------------------------------
CMD uv run --package gltestserver gltestserver run --directory ${REPO_PATH}/.gltestserver
