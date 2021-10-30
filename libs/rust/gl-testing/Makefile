PROTOC_OPTS=-I../../proto --python_out=gltesting --grpc_python_out=gltesting

PROTOS= \
	gltesting/scheduler_pb2.py \
	gltesting/scheduler_pb2_grpc.py \
	gltesting/greenlight_pb2.py \
	gltesting/greenlight_pb2_grpc.py

all: ${PROTOS}

gltesting/scheduler_pb2.py gltesting/scheduler_pb2_grpc.py: ../../proto/scheduler.proto
	python -m grpc_tools.protoc ${PROTOC_OPTS} scheduler.proto
	# Postprocess the import path
	sed -i 's/import scheduler_pb2 as scheduler__pb2/from . import scheduler_pb2 as scheduler__pb2/g' gltesting/scheduler_pb2_grpc.py

gltesting/greenlight_pb2.py gltesting/greenlight_pb2_grpc.py: ../../proto/greenlight.proto
	python -m grpc_tools.protoc ${PROTOC_OPTS} greenlight.proto
	# Postprocess the import path
	sed -i 's/import greenlight_pb2 as greenlight__pb2/from . import greenlight_pb2 as greenlight__pb2/g' gltesting/greenlight_pb2_grpc.py

clean:
	rm -f ${PROTOS}
