PROTOC_OPTS=-I../../proto --python_out=glclient

PROTOS= \
	glclient/scheduler_pb2.py \
	glclient/greenlight_pb2.py \
	glclient/greenlight_pb2_grpc.py

all: ${PROTOS}

glclient/scheduler_pb2.py: ../../proto/scheduler.proto
	python -m grpc_tools.protoc ${PROTOC_OPTS} scheduler.proto

glclient/greenlight_pb2.py glclient/greenlight_pb2_grpc.py: ../../proto/greenlight.proto
	python -m grpc_tools.protoc ${PROTOC_OPTS} --grpc_python_out=glclient greenlight.proto
	# Postprocess the import path
	sed -i 's/import greenlight_pb2 as greenlight__pb2/from . import greenlight_pb2 as greenlight__pb2/g' glclient/greenlight_pb2_grpc.py

clean:
	rm -f ${PROTOS}
