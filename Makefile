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

ARTIFACTS = \
	.coverage

CHANGED_RUST_SOURCES=$(shell git diff --name-only origin/main | grep '\.rs')

# Variable to collect all generated files into, so we can clean and
# rebuild them easily.
GENALL =

DOCKER_OPTIONS= \
	--rm \
	--user $(shell id -u):$(shell id -g) \
	-e TMPDIR=/tmp/gltesting/tmp \
	-v /tmp/gltesting/tmp:/tmp/gltesting/tmp \
	-e CARGO_TARGET_DIR=/tmp/gltesting/target/target \
	-v /tmp/gltesting/target:/tmp/gltesting/target \
	-v /tmp/gltesting/cargo/registry:/opt/cargo/registry \
	-v ${REPO_ROOT}:/repo

.PHONY: ensure-docker build-self check-self docker-image docs wheels

include libs/gl-client/Makefile
include libs/gl-client-py/Makefile
include libs/gl-testing/Makefile

check: check-rs check-py

clean: clean-rs
	rm -rf ${ARTIFACTS}
	rm ${GENALL}

gen: ${GENALL}

build-self: ensure-docker
	cargo build --all
	cd libs/gl-client-py; maturin develop
	mypy examples/python

check-all: check-self check-self-gl-client check-py

check-self: ensure-docker build-self
	PYTHONPATH=/repo/libs/gl-testing \
	pytest -vvv \
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
	mypy examples/python
	cargo build --manifest-path=./examples/rust/getting-started/Cargo.toml
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
