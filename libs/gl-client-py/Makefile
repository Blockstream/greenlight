# Execute only from the repo root
REPO=$(shell git rev-parse --show-toplevel)
DIR=${REPO}/libs/gl-client-py
PYVERSION=$(shell cd libs && cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "gl-client-py") | .version')
PYDIR=${REPO}/libs/gl-client-py

PYPROTOC_OPTS = \
	-Ilibs/proto \
	--python_out=${PYDIR}/glclient \
	--grpc_python_out=${PYDIR}/glclient \
	--experimental_allow_proto3_optional

PYPROTOS = \
	${PYDIR}/glclient/scheduler_pb2.py \
	${PYDIR}/glclient/greenlight_pb2.py \
	${PYDIR}/glclient/node_pb2.py \
	${PYDIR}/glclient/scheduler_pb2_grpc.py \
	${PYDIR}/glclient/greenlight_pb2_grpc.py \
	${PYDIR}/glclient/node_pb2_grpc.py

PROTOSRC = \
	libs/proto/scheduler.proto \
	libs/proto/greenlight.proto \
	libs/proto/node.proto

pygrpc: ${PYPROTOS}

${PYPROTOS}: ${PROTOSRC}
	python -m grpc_tools.protoc ${PYPROTOC_OPTS} scheduler.proto
	python -m grpc_tools.protoc ${PYPROTOC_OPTS} greenlight.proto
	python -m grpc_tools.protoc ${PYPROTOC_OPTS} node.proto
	# Postprocess the import path
	sed -i 's/import scheduler_pb2 as scheduler__pb2/from . import scheduler_pb2 as scheduler__pb2/g' ${PYDIR}/glclient/scheduler_pb2_grpc.py
	sed -i 's/import greenlight_pb2 as greenlight__pb2/from . import greenlight_pb2 as greenlight__pb2/g' ${PYDIR}/glclient/greenlight_pb2_grpc.py
	sed -i 's/import node_pb2 as node__pb2/from . import node_pb2 as node__pb2/g' ${PYDIR}/glclient/node_pb2_grpc.py

check-py:
	cd libs/gl-client-py; pytest tests -n $(shell nproc)

clean-py:
	rm -f ${PROTOS} ${PYDIR}/build ${PYDIR}/dist

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
	sudo docker buildx build \
	  --load \
	  -t gl-client-py-builder \
	  --build-arg DOCKER_USER=$(shell whoami) \
	  ${PYDIR}

	sudo docker run -ti \
          -v ${REPO}:/io \
	  gl-client-py-builder

${PYDIR}/pyproject.toml: ${PYDIR}/Cargo.toml
	cd ${PYDIR}; poetry version ${PYVERSION}

# Pickup the wheels in ${REPO}/wheelhouse and send them to PyPI
publish-py: build-py
	cd tools/glcli; poetry build --no-interaction
	cd ${PYDIR}; poetry publish --skip-existing


package-py: build-py
