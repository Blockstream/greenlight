ifdef GL_DOCKER
REPO_ROOT=/repo
else
REPO_ROOT=$(shell git rev-parse --show-toplevel)
endif

LIBS=${REPO_ROOT}/libs
SELF_DIR=${LIBS}/gl-plugin

MANIFEST_PATH = --manifest-path=${SELF_DIR}/Cargo.toml
FMT_OPTS = -- --check
CLIPPY_OPTS = --no-deps --message-format short

# Check code for formatting issues
fmt:
	cargo fmt ${MANIFEST_PATH} ${FMT_OPTS}

# Fix formatting issues
fmt-fix:
	cargo fmt ${MANIFEST_PATH}

# Check Clippy linter, does not fail on warnings
clippy:
	cargo clippy ${CLIPPY_OPTS} ${MANIFEST_PATH}

# Run clippy linter and fix issues automatically where possible
clippy-fix:
	cargo clippy --fix ${MANIFEST_PATH}

# Run tests
test:
	cargo test ${MANIFEST_PATH}

# Checks that clippy and tests only produce warnings
check: clippy test

# Same as check but also fails if there are formatting issues
check-all: fmt clippy test

# Syncronize proto and tls files into .resources dir
sync-files:
	rsync -avr --delete --delete-excluded ${LIBS}/proto/glclient/greenlight.proto ${SELF_DIR}/.resources/proto/glclient/greenlight.proto
	git add ${SELF_DIR}/.resources