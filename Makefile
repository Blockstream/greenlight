REPO_ROOT=$(shell git rev-parse --show-toplevel)
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

.PHONY: ensure-docker build-self check-self docker-image docs wheels

include libs/gl-client/Makefile
include libs/gl-client-py/Makefile
include libs/gl-client-js/Makefile

check: check-rs check-py check-js

clean: clean-rs
	rm -rf ${ARTIFACTS}

build-self: ensure-docker
	(cd libs; cargo build --all)
	(cd libs/gl-client-py && \
	maturin build --strip && \
	pip install --force-reinstall /tmp/target/wheels/gl_client_py*.whl)
	pip install coverage

check-self: ensure-docker
	PYTHONPATH=/repo/libs/gl-testing pytest -vvv /repo/libs/gl-testing ${PYTEST_OPTS}

ensure-docker:
	@if [ "x${GL_DOCKER}" != "x1" ]; then \
		echo "We are not running in the gl-testing docker container, refusing to run"; \
		exit 1; \
	fi
docker-image: ${REPO_ROOT}/libs/gl-testing/Dockerfile
	docker build -t gltesting -f libs/gl-testing/Dockerfile .

docker-shell:
	docker run \
	  -ti \
	  --net=host \
	  --rm \
          --cap-add=SYS_PTRACE \
	  -e TMPDIR=/tmp/gltesting/ \
	  -v /tmp/gltesting/:/tmp/gltesting \
          -e CARGO_TARGET_DIR=/tmp/target \
          -v /tmp/target:/tmp/target \
          -v /tmp/gl-cargo-registry:/root/.cargo/registry/ \
	  -v ${REPO_ROOT}:/repo \
	  gltesting bash

docker-check-self:
	docker run \
	  -t \
	  --rm \
	  -v ${REPO_ROOT}:/repo \
	  gltesting make check-self

docker-check:
	docker run \
	  -t \
	  --rm \
	  -v ${REPO_ROOT}:/repo \
	  gltesting make check

CLN_VERSIONS = \
	v0.10.1 \
	v0.10.2 \
	v0.11.0.1 \
	v0.11.2gl2 \
	v0.11.2 \
	v22.11gl1

CLN_TARGETS = $(foreach VERSION,$(CLN_VERSIONS),cln-versions/$(VERSION)/usr/local/bin/lightningd)

cln-versions/%/usr/local/bin/lightningd: cln-versions/lightningd-%.tar.bz2
	@echo "Extracting $* from tarball $< into cln-versions/$*/"
	mkdir -p "cln-versions/$*"
	tar -xjf $< -C "cln-versions/$*/"

cln-versions/lightningd-%.tar.bz2:
	mkdir -p cln-versions
	wget -q "https://storage.googleapis.com/greenlight-artifacts/cln/lightningd-$*.tar.bz2" -O $@


cln: ${CLN_TARGETS}

DOCSECRET="2dijIFEFSh/"
docs:
	mkdir -p ${REPO_ROOT}/site/2dijIFEFSh
	(cd docs; mkdocs build --strict --clean --site-dir=${REPO_ROOT}/site/${DOCSECRET} --verbose)
	pdoc -o site/${DOCSECRET}py glclient
	ghp-import ${REPO_ROOT}/site -n -m "Deploy docs" --push --branch gh-pages --remote origin
