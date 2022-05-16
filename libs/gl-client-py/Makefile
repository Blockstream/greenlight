PROTOC_OPTS=-I../proto --python_out=glclient --grpc_python_out=glclient --experimental_allow_proto3_optional

PROTOS= \
	glclient/scheduler_pb2.py \
	glclient/greenlight_pb2.py \
	glclient/scheduler_pb2_grpc.py \
	glclient/greenlight_pb2_grpc.py

UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Linux)
  OS = linux
endif
ifeq ($(UNAME_S),Darwin)
  OS = macos
endif

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
	cd libs/gl-client-py; pytest tests -n $(shell nproc)

clean:
	rm -f ${PROTOS} build dist

# Linux builds are performed in a docker container so we link against
# an oldish glibc, increasing portability.
build-linux:
	docker build -t gl-client-py-builder .
	docker run -v $(shell pwd)/..:/io gl-client-py-builder

test: build
	virtualenv .tmpenv --python=$(which python3) --download --always-copy --clear
	.tmpenv/bin/python3 -m pip install -U pip wheel pytest pytest-timeout
	.tmpenv/bin/python3 -m pip install -f ../dist gl-client-py ../gl-testing
	cd .tmpenv && bin/pytest ../tests
	rm -rf .tmpenv

build-macos:
	# Build with `cargo` first so we don't get stuck on the `mako` issue in libhsmd-sys-rs
	PKG_CONFIG_SYSROOT_DIR=/ cargo build --profile=release --target aarch64-apple-darwin
	PKG_CONFIG_SYSROOT_DIR=/ maturin build --profile=release --target aarch64-apple-darwin --find-interpreter --out=$(shell pwd)/../dist

	# Build with `cargo` first so we don't get stuck on the `mako` issue in libhsmd-sys-rs
	PKG_CONFIG_SYSROOT_DIR=/ cargo build --profile=release --target x86_64-apple-darwin
	PKG_CONFIG_SYSROOT_DIR=/ maturin build --profile=release --target x86_64-apple-darwin --find-interpreter --out=$(shell pwd)/../dist

build: build-${OS}
	maturin sdist --out ../dist

sdist:
	python setup.py sdist

publish: sdist
	twine upload --repository-url https://us-west2-python.pkg.dev/c-lightning/greenlight-pypi/ dist/gl-client-*.tar.gz
