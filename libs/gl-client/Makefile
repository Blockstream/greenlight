ifdef GL_DOCKER
REPO_ROOT=/repo
else
REPO_ROOT=$(shell git rev-parse --show-toplevel)
endif

LIBS=${REPO_ROOT}/libs
SELF_DIR=${LIBS}/gl-client

check-gl-client:
	(cd ${GL_CLIENT_DIR}; cargo test)

# Syncronize proto and tls files into .resources dir
sync-files:
	rsync -avr --delete --delete-excluded ${LIBS}/proto/ ${SELF_DIR}/.resources/proto
	rsync -avr --delete --delete-excluded ${LIBS}/tls/ ${SELF_DIR}/.resources/tls
	git add ${SELF_DIR}/.resources

