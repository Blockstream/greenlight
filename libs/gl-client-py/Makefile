PROTOC_OPTS=-I../proto --python_out=glclient --grpc_python_out=glclient --experimental_allow_proto3_optional

PROTOS= \
	glclient/scheduler_pb2.py \
	glclient/greenlight_pb2.py \
	glclient/scheduler_pb2_grpc.py \
	glclient/greenlight_pb2_grpc.py

all: ${PROTOS}

glclient/scheduler_pb2.py glclient/scheduler_pb2_grpc.py: ../proto/scheduler.proto
	python -m grpc_tools.protoc ${PROTOC_OPTS} scheduler.proto
	# Postprocess the import path
	sed -i 's/import scheduler_pb2 as scheduler__pb2/from . import scheduler_pb2 as scheduler__pb2/g' glclient/scheduler_pb2_grpc.py
glclient/greenlight_pb2.py glclient/greenlight_pb2_grpc.py: ../proto/greenlight.proto
	python -m grpc_tools.protoc ${PROTOC_OPTS} greenlight.proto
	# Postprocess the import path
	sed -i 's/import greenlight_pb2 as greenlight__pb2/from . import greenlight_pb2 as greenlight__pb2/g' glclient/greenlight_pb2_grpc.py

check-py:
	cd libs/gl-client-py; pytest tests -n 5

clean-py:
	rm -f ${PROTOS}
