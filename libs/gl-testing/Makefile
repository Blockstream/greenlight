ifdef GL_DOCKER
REPO_ROOT=/repo
else
REPO_ROOT=$(shell git rev-parse --show-toplevel)
endif

TESTINGDIR=${REPO_ROOT}/libs/gl-testing

TESTPROTOC_OPTS = \
	-I ${REPO_ROOT}/libs/proto \
	--purerpc_out=${TESTINGDIR}/gltesting \
	--experimental_allow_proto3_optional

GEN_TESTING = \
	${TESTINGDIR}/gltesting/scheduler_grpc.py

GENALL += ${GEN_TESTING}

${TESTINGDIR}/gltesting/scheduler_grpc.py: testgrpc

check-testing-py:
	cd ${TESTINGDIR}; uv run pytest tests -n 4 -vvv

testgrpc: ${REPO_ROOT}/libs/proto/glclient/scheduler.proto
	uv run python3 -m grpc_tools.protoc ${TESTPROTOC_OPTS} glclient/scheduler.proto
	mv ${TESTINGDIR}/gltesting/glclient/scheduler_grpc.py ${TESTINGDIR}/gltesting/scheduler_grpc.py
	rm -rf ${TESTINGDIR}/gltesting/glclient

protoc:
	uv run python3 -m grpc_tools.protoc -I. --python_out=. --pyi_out=. --purerpc_out=. --grpc_python_out=. gltesting/test.proto

