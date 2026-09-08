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
	#cd ${TESTINGDIR}; mypy gltesting
	cd ${TESTINGDIR}; poetry install --with=dev; poetry run pytest tests -n 4 -vvv

# Pinned and isolated for the same reason as gl-client-py's PROTOC_TOOLCHAIN:
# the generated stubs must not depend on whose venv produced them. Keep the
# grpcio-tools version in step with that one.
TESTPROTOC_TOOLCHAIN = \
	uv run --no-project \
	--with grpcio-tools==1.81.1 \
	--with purerpc \
	python -m grpc_tools.protoc

testgrpc: ${REPO_ROOT}/libs/proto/glclient/scheduler.proto
	${TESTPROTOC_TOOLCHAIN} ${TESTPROTOC_OPTS} glclient/scheduler.proto
	mv ${TESTINGDIR}/gltesting/glclient/scheduler_grpc.py ${TESTINGDIR}/gltesting/scheduler_grpc.py
	rm -rf ${TESTINGDIR}/gltesting/glclient

# Uses the same pinned, isolated toolchain as testgrpc so the checked-in
# stubs do not depend on the ambient venv.
protoc:
	cd ${TESTINGDIR}; ${TESTPROTOC_TOOLCHAIN} -I. --python_out=. --pyi_out=. --purerpc_out=. --grpc_python_out=. gltesting/test.proto

