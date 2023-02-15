REPO_ROOT=$(shell git rev-parse --show-toplevel)

include libs/gl-client-py/Makefile
include libs/gl-client-js/Makefile

.PHONY: ensure-docker build-self check-self docker-image

check: check-rs check-py check-js

check-rs:
	cd libs; cargo test --all -- --test-threads=1

clean-rs:
	cd libs; cargo clean

clean: clean-rs

build-self: ensure-docker
	(cd libs; cargo build --all)
	(cd libs/gl-client-py && \
	maturin build --strip && \
	pip install --force-reinstall /tmp/target/wheels/gl_client_py*.whl)
	pip install coverage

check-self: ensure-docker
	PYTHONPATH=/repo/libs/gl-testing pytest -vvv /repo/libs/gl-testing -n=$(shell nproc)

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
	  --rm \
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
