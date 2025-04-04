.PHONY: check-rustfmt

ifdef GL_DOCKER
REPO_ROOT=/repo
else
REPO_ROOT=$(shell git rev-parse --show-toplevel)
endif

UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -p)

ifeq ($(UNAME_S),Linux)
  OS = linux
endif
ifeq ($(UNAME_S),Darwin)
  OS = macos
endif

ifeq (${UNAME_M},arm64)
  RSARCH = "aarch64"
else
  RSARCH = ${UNAME_M}
endif

# Path to workspace crates
LIBS = ${REPO_ROOT}/libs
GL_CLIENT = ${LIBS}/gl-client
GL_PLUGIN = ${LIBS}/gl-plugin
GL_SIGNERPROXY = ${LIBS}/gl-signerproxy

# Do not run clippy on dependencies
CLIPPY_OPTS = --no-deps --message-format short

ARTIFACTS = \
	.coverage

CHANGED_RUST_SOURCES=$(shell git diff --name-only origin/main | grep '\.rs')

# Variable to collect all generated files into, so we can clean and
# rebuild them easily.
GENALL =

CLN_VERSIONS = \
	v0.10.1 \
	v0.10.2 \
	v0.11.0.1 \
	v0.11.2gl2 \
	v0.11.2 \
	v22.11gl1 \
	v23.05gl1 \
	v23.08gl1 \
	v24.02gl1 \
	v24.11gl1

DOCKER_OPTIONS= \
	--rm \
	--user $(shell id -u):$(shell id -g) \
	-e TMPDIR=/tmp/gltesting/tmp \
	-v /tmp/gltesting/tmp:/tmp/gltesting/tmp \
	-e CARGO_TARGET_DIR=/tmp/gltesting/target/target \
	-v /tmp/gltesting/target:/tmp/gltesting/target \
	-v /tmp/gltesting/cargo/registry:/opt/cargo/registry \
	-v ${REPO_ROOT}:/repo

include libs/gl-client-py/Makefile
include libs/gl-testing/Makefile

# sync-files section
.PHONY: sync-files gl_client_sync-files gl_plugin_sync-files gl_signerproxy_sync-files
gl_client_sync-files:
	$(MAKE) -C ${GL_CLIENT} sync-files

gl_plugin_sync-files:
	$(MAKE) -C ${GL_PLUGIN} sync-files

gl_signerproxy_sync-files:
	$(MAKE) -C ${GL_SIGNERPROXY} sync-files

# Sync all files
sync-files: gl_client_sync-files gl_plugin_sync-files gl_signerproxy_sync-files

# Run clippy
clippy:
	cargo clippy ${CLIPPY_OPTS}

# Run rust tests
test-rs:
	cargo test

# Check runs clippy and the tests but does not fail on clippy warnings
check-rs: clippy test-rs

.PHONY: clippy test-rs check-rs ensure-docker build-self check-self check-all docker-image docs wheels

clean-rs:
	cargo clean

clean: clean-rs
	rm -rf ${ARTIFACTS}
	rm ${GENALL}

gen: ${GENALL}

build-self: ensure-docker
	cargo build --all
	uv sync --package gl-client --group dev

check-all: check-rs check-self check-py check-testing-py

check-self: ensure-docker build-self
	uv sync --package gl-testing
	uv pip install libs/cln-version-manager
	ls -lha .venv/bin
	PYTHONPATH=/repo/libs/gl-testing \
	GL_TESTING_IGNORE_HASH=1 \
	uv run python3 -m pytest -vvv -n 4 \
	  /repo/libs/gl-testing \
	  ${PYTEST_OPTS}

check-rustfmt:
	@if [ -n "${CHANGED_RUST_SOURCES}" ]; then \
		rustfmt --edition 2021 --check ${CHANGED_RUST_SOURCES}; else \
		echo "skip rustfmt check no changes detected ${CHANGED_RUST_SOURCES}"; \
	fi

ensure-docker:
	@if [ "x${GL_DOCKER}" != "x1" ]; then \
		echo "We are not running in the gl-testing docker container, refusing to run"; \
		exit 1; \
	fi

docker-image: ${REPO_ROOT}/docker/gl-testing/Dockerfile
	docker buildx build \
	  --load \
	  --build-arg DOCKER_USER=$(shell whoami) \
	  --build-arg UID=$(shell id -u) \
	  --build-arg GID=$(shell id -g) \
	  --build-arg GL_TESTING_IGNORE_HASH=1 \
	  -t gltesting \
	  -f docker/gl-testing/Dockerfile \
	  .

docker-volumes:
	mkdir -p /tmp/gltesting/tmp && \
	mkdir -p /tmp/gltesting/target &&\
	mkdir -p /tmp/gltesting/cargo/registry \

docker-shell: docker-volumes
	docker run \
		-ti \
		--cap-add=SYS_PTRACE \
		${DOCKER_OPTIONS} \
		gltesting bash

docker-check-self: docker-volumes
	docker run \
	  -t \
	  ${DOCKER_OPTIONS} \
	  gltesting make build-self check-self

docker-check-all:docker-volumes
	docker run \
	  -t \
	  ${DOCKER_OPTIONS} \
	  gltesting make build-self check-all

docker-check: docker-volumes
	docker run \
	  -t \
	  -v ${REPO_ROOT}:/repo \
	  gltesting make check

docker-check-py: docker-volumes
	docker run \
		-t \
		${DOCKER_OPTIONS} \
		gltesting make build-self check-py

cln: ${CLN_TARGETS}

docs:
	# TODO mypy fails to verify the generated primitives_pb2 types
	mypy examples/python/snippets
	cargo build --manifest-path=./examples/rust/Cargo.toml
	mkdir -p ${REPO_ROOT}/site/
	(cd docs; mkdocs build --strict --clean --site-dir=${REPO_ROOT}/site/ --verbose)
	pdoc -o site/py glclient

docs-publish: docs
	ghp-import ${REPO_ROOT}/site \
	  --no-jekyll \
	  -m "Deploy docs [skip ci]" \
	  --force \
	  --no-history \
	  --push \
	  --branch gh-pages \
	  --remote origin

gltestserver-image: ${REPO_ROOT}/docker/gl-testserver/Dockerfile
	docker buildx build \
	  --load \
	  --build-arg DOCKER_USER=$(shell whoami) \
	  --build-arg UID=$(shell id -u) \
	  --build-arg GID=$(shell id -g) \
	  --build-arg REPO_PATH=$(shell git rev-parse --show-toplevel) \
	  -t gltestserver \
	  -f docker/gl-testserver/Dockerfile \
	  .

gltestserver: gltestserver-image
	docker run \
	  --rm \
	  --user $(shell id -u):$(shell id -g) \
	  -e DOCKER_USER=$(shell whoami) \
	  --net=host \
	  -ti \
	  -v $(shell pwd)/:$(shell pwd) \
	  gltestserver
