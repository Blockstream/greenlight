# Execute only from the repo root
ifdef GL_DOCKER
REPO=/repo
else
REPO=$(shell git rev-parse --show-toplevel)
endif

DIR=${REPO}/libs/gl-client-py
PYVERSION=$(shell cd libs && cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "gl-client-py") | .version')
PYDIR=${REPO}/libs/gl-client-py
PROTODIR=${REPO}/libs/proto

PYPROTOC_OPTS = \
	-I ${PROTODIR} \
	--python_out=. \
	--grpc_python_out=. \
	--mypy_out=. \
	--experimental_allow_proto3_optional

PYPROTOS = \
	${PYDIR}/glclient/greenlight_pb2.py \
	${PYDIR}/glclient/greenlight_pb2_grpc.py \
	${PYDIR}/glclient/scheduler_pb2.py \
	${PYDIR}/glclient/scheduler_pb2_grpc.py

PROTOSRC = \
	${PROTODIR}/glclient/scheduler.proto \
	${PROTODIR}/glclient/greenlight.proto \

GENALL += ${PYPROTOS}

${PYPROTOS}: pygrpc

pygrpc: ${PROTOSRC}
	python -m grpc_tools.protoc ${PYPROTOC_OPTS} glclient/scheduler.proto
	python -m grpc_tools.protoc ${PYPROTOC_OPTS} glclient/greenlight.proto

check-py:
	cd ${PYDIR}; mypy glclient
	cd libs/gl-client-py; pytest tests -n $(shell nproc)

clean-py:
	rm -f ${PYPROTOS} ${PYDIR}/build ${PYDIR}/dist

test: build
	virtualenv .tmpenv --python=$(which python3) --download --always-copy --clear
	.tmpenv/bin/python3 -m pip install -U pip wheel pytest pytest-timeout
	.tmpenv/bin/python3 -m pip install -f ${PYDIR}/../dist gl-client-py ${PYDIR}/../gl-testing
	cd .tmpenv && bin/pytest ${PYDIR}/../tests
	rm -rf .tmpenv

build-py-macos:
	cd ${PYDIR}/; \
	maturin build \
	  --release \
	  --target aarch64-apple-darwin \
	  --out=${PYDIR}/dist

	cd ${PYDIR}; \
	maturin build \
	  --release \
	  --target x86_64-apple-darwin \
	  --out=${PYDIR}/dist

build-py: ${PYDIR}/pyproject.toml build-py-${OS}

# build-py-linux builds the wheels and sdist package in docker and
# drops them into ${REPO}/wheelhouse. The docker build is needed to
# get an old(-ish) glibc, maximizing compatibility.
build-py-linux:
	docker buildx build \
	  --load \
	  -t gl-client-py-builder \
	  --build-arg DOCKER_USER=$(shell whoami) \
	  ${PYDIR}

	docker run -ti \
          -v ${REPO}:/io \
	  gl-client-py-builder

${PYDIR}/pyproject.toml: ${PYDIR}/Cargo.toml
	cd ${PYDIR}; poetry version ${PYVERSION}

# Pickup the wheels in ${REPO}/wheelhouse and send them to PyPI
publish-py: build-py
	cd tools/glcli; poetry build --no-interaction
	cd ${PYDIR}; poetry publish --skip-existing


package-py: build-py
