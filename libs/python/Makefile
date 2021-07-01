PROTOC_OPTS=-Iproto --python_out=. --grpc_python_out=.
GRPC_FILES = \
	glapi/scheduler_pb2.py

glapi/scheduler_pb2_grpc.py glapi/scheduler_pb2.py: proto/glapi/scheduler.proto
	python -m grpc_tools.protoc ${PROTOC_OPTS} proto/glapi/scheduler.proto

glapi/greenlight_pb2_grpc.py glapi/greenlight_pb2.py: proto/glapi/greenlight.proto
	python -m grpc_tools.protoc ${PROTOC_OPTS} proto/glapi/greenlight.proto


pygrpc:
	cp ../../proto/greenlight.proto proto/glapi
	cp ../../proto/scheduler.proto proto/glapi
	python -m grpc_tools.protoc ${PROTOC_OPTS} proto/glapi/greenlight.proto
	python -m grpc_tools.protoc ${PROTOC_OPTS} proto/glapi/scheduler.proto
test:
	virtualenv .tmpenv
	(cd .tmpenv; bin/pip3 install -U pip setuptools wheel pep517)
	(cd .tmpenv; bin/pip3 install ../dist/glapi-0.0.1.tar.gz)
	(cd .tmpenv; bin/python3 -c "from glapi import libhsmd;import _libhsmd;print(libhsmd.__file__);print(_libhsmd.__file__);print(libhsmd.init('00'*32, 'bitcoin'));req='0017000B48656c6c6f20776f726c64';sig = libhsmd.handle(1024, 0, None, req);assert sig.startswith('007b');assert len(sig) / 2 == 67")
	.tmpenv/bin/python3 -c 'import _libhsmd;print(_libhsmd.__file__)'
	find .tmpenv -name '_libhsmd.*.so' | xargs ldd
	rm -rf .tmpenv

clean:
	rm -rf build dist glapi.egg-info .mypy_cache src tmpenv .tmpenv
	rm -f setup.py
