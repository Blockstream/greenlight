TESTINGDIR=${REPO}/libs/gl-testing

TESTPROTOC_OPTS = \
	-I ${REPO}/libs/proto \
	--purerpc_out=${TESTINGDIR}/gltesting \
	--experimental_allow_proto3_optional

GEN_TESTING = \
	${TESTINGDIR}/gltesting/scheduler_grpc.py

GENALL += ${GEN_TESTING}

${TESTINGDIR}/gltesting/scheduler_grpc.py: testgrpc

testgrpc: ${REPO}/libs/proto/glclient/scheduler.proto
	python -m grpc_tools.protoc ${TESTPROTOC_OPTS} glclient/scheduler.proto
	mv ${TESTINGDIR}/gltesting/glclient/scheduler_grpc.py ${TESTINGDIR}/gltesting/scheduler_grpc.py
	rm -rf ${TESTINGDIR}/gltesting/glclient



