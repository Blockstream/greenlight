ifdef GL_DOCKER
REPO_ROOT=/repo
else
REPO_ROOT=$(shell git rev-parse --show-toplevel)
endif

LIBS=${REPO_ROOT}/libs
SELF_DIR=${LIBS}/gl-plugin

# Syncronize proto and tls files into .resources dir
sync-files:
	rsync -avr --delete --delete-excluded ${LIBS}/proto/glclient/greenlight.proto ${SELF_DIR}/.resources/proto/glclient/greenlight.proto
	git add ${SELF_DIR}/.resources