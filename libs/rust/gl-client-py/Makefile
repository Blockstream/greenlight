PROTOC_OPTS=-I../../proto --python_out=glclient

PROTOS= \
	glclient/scheduler_pb2.py \
	glclient/greenlight_pb2.py \
	glclient/greenlight_pb2_grpc.py

all: ${PROTOS}

glclient/scheduler_pb2.py: ../../proto/scheduler.proto
	python -m grpc_tools.protoc ${PROTOC_OPTS} scheduler.proto
	# Postprocess the import path
	sed -i 's/import scheduler_pb2 as scheduler__pb2/from . import scheduler_pb2 as scheduler__pb2/g' glclient/scheduler_pb2_grpc.py

glclient/greenligh_pb2.py: ../../proto/greenlight.proto
	python -m grpc_tools.protoc ${PROTOC_OPTS} greenlight.proto

clean:
	rm -f ${PROTOS}
