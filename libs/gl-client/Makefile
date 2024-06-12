ifdef GL_DOCKER
REPO_ROOT=/repo
else
REPO_ROOT=$(shell git rev-parse --show-toplevel)
endif

LIBS=${REPO}/libs
GL_CLIENT_DIR=${LIBS}/gl-client

check-gl-client:
	(cd ${GL_CLIENT_DIR}; cargo test)

sync-files:
	rm -rf ${GL_CLIENT_DIR}/.resource/proto .resource/tls
	rsync -avr ${LIBS}/proto/ ${GL_CLIENT_DIR}/.resources/proto
	rsync -avr ${LIBS}/tls/ ${GL_CLIENT_DIR}/.resources/tls
	git add ${GL_CLIENT_DIR}/.resources
