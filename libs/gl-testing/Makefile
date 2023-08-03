TESTINGDIR=${REPO}/libs/gl-testing

TESTPROTOC_OPTS = \
	-I${REPO}/libs/proto \
	--python_out=${TESTINGDIR}/gltesting \
	--purerpc_out=${TESTINGDIR}/gltesting \
	--experimental_allow_proto3_optional

GEN_TESTING = \
	${TESTINGDIR}/gltesting/scheduler_grpc.py

GENALL += ${GEN_TESTING}

testgrpc ${TESTINGDIR}/gltesting/scheduler_grpc.py: ${REPO}/libs/proto/scheduler.proto
	python -m grpc_tools.protoc ${TESTPROTOC_OPTS} ${REPO}/libs/proto/scheduler.proto
	sed -E -i 's/import (.+)_pb2 as .+__pb2/from glclient import \1_pb2 as \1__pb2/g' ${TESTINGDIR}/gltesting/scheduler_grpc.py

