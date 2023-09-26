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
	v23.08gl1

CLN_TARGETS = $(foreach VERSION,$(CLN_VERSIONS),cln-versions/$(VERSION)/usr/local/bin/lightningd)

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
	(cd libs; cargo build --all)
	(cd libs/gl-client-py && python3 -m maturin develop)
	pip install -e libs/gl-testing

check-all: check-self check-self-gl-client check-py

check-self: ensure-docker
	PYTHONPATH=/repo/libs/gl-testing \
	pytest -vvv \
	  /repo/libs/gl-testing \
	  ${PYTEST_OPTS}

ensure-docker:
	@if [ "x${GL_DOCKER}" != "x1" ]; then \
		echo "We are not running in the gl-testing docker container, refusing to run"; \
		exit 1; \
	fi

docker-image: ${REPO_ROOT}/libs/gl-testing/Dockerfile
	docker buildx build \
	  --load \
	  --build-arg DOCKER_USER=$(shell whoami) \
	  --build-arg UID=$(shell id -u) \
	  --build-arg GID=$(shell id -g) \
	  -t gltesting \
	  -f libs/gl-testing/Dockerfile \
	  .

docker-shell:
	mkdir -p /tmp/gltesting/tmp && \
	mkdir -p /tmp/gltesting/target && \
	mkdir -p /tmp/gltesting/.cargo/.registry && \
	docker run \
		-ti \
		--net=host \
		--rm \
		--cap-add=SYS_PTRACE \
		-e TMPDIR=/tmp/gltesting/tmp \
		-v /tmp/gltesting/:/tmp/gltesting \
		-e CARGO_TARGET_DIR=/tmp/gltesting/target \
		-v /tmp/gltesting/.cargo/.registry:/home/$(shell whoami)/.cargo/registry/ \
		-v ${REPO_ROOT}:/repo \
		gltesting bash

docker-check-self:
	docker run \
	  -t \
	  --rm \
	  -v ${REPO_ROOT}:/repo \
	  gltesting make build-self check-self

docker-check-all:
	docker run \
	  -t \
	  --rm \
	  -v ${REPO_ROOT}:/repo \
	  gltesting make build-self check-all

docker-check:
	docker run \
	  -t \
	  --rm \
	  -v ${REPO_ROOT}:/repo \
	  gltesting make check

docker-check-py:
	docker run \
		-t \
		--rm \
		-v ${REPO_ROOT}:/repo \
		gltesting make build-self check-py

cln-versions/%/usr/local/bin/lightningd: cln-versions/lightningd-%.tar.bz2
	@echo "Extracting $* from tarball $< into cln-versions/$*/"
	mkdir -p "cln-versions/$*"
	tar -xjf $< -C "cln-versions/$*/"

cln-versions/lightningd-%.tar.bz2:
	mkdir -p cln-versions
	wget -q "https://storage.googleapis.com/greenlight-artifacts/cln/lightningd-$*.tar.bz2" -O $@


cln: ${CLN_TARGETS}

docs:
	mkdir -p ${REPO_ROOT}/site/
	(cd docs; mkdocs build --strict --clean --site-dir=${REPO_ROOT}/site/ --verbose)
	pdoc -o site/py glclient
	ghp-import ${REPO_ROOT}/site \
	  --no-jekyll \
	  -m "Deploy docs [skip ci]" \
	  --force \
	  --no-history \
	  --push \
	  --branch gh-pages \
	  --remote origin
