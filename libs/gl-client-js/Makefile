ifdef GL_DOCKER
REPO=/repo
else
REPO=$(shell git rev-parse --show-toplevel)
endif

ARCH=$(shell node -e "console.log(process.arch)")
PLAT=$(shell uname -s | tr '[:upper:]' '[:lower:]')
JSDIR=${REPO}/libs/gl-client-js
VERSION=$(shell jq '.version' ${JSDIR}/package.json -r)
JSVERSION=$(shell cd libs && cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "gl-client") | .version')
JSSRC=${JSDIR}/index.js ${JSDIR}/index.test.js
JSRSSRC=$(shell find ${JSDIR} -name '*.rs')

PACKAGE=${JSDIR}/build/stage/${VERSION}/gl-client-${VERSION}-${PLAT}-${ARCH}.tar.gz

.PHONY: build-js-macos-${ARCH} build-js-macos build-js-linux build-js
.NOTPARALLEL: build-js-macos-arm64 build-js-macos-x86_64 build-js-linux

JSPROTOS = \
	${REPO}/libs/proto/scheduler.proto \
	${REPO}/libs/proto/greenlight.proto

jsproto: ${JSDIR}/proto.js

${JSDIR}/proto.js: node_modules ${JSPROTOS}
	cd ${JSDIR} && \
	npx pbjs -t static-module -w commonjs -o ${JSDIR}/proto.js ${JSPROTOS}

build-js-macos-${ARCH}: ${JSSRC} ${JSRSSRC} 
	cd ${JSDIR} && \
	TARGET=${RSARCH}-apple-darwin npm run build && \
	mv index.node index-${VERSION}-${ARCH}-apple-darwin.node

build-js-macos: build-js-macos-${ARCH}

# Installs the js dependencies.
node_modules:
	cd ${JSDIR}; npm install

# We build the binary in an old(-ish) ubuntu so we don't rely on new
# `glibc`
docker-build-js-linux: ${JSSRC} ${JSRSSRC}
	echo "Building linux"
	cd libs/gl-client-js && \
	TARGET=${ARCH}-unknown-linux-gnu npm run build && \
	mv index.node index-${VERSION}-${ARCH}-unknown-linux-gnu.node

# Build the docker container and then run the inner target in it.
build-js-linux: ${JSSRC} ${JSRSSRC}
	sudo docker buildx build \
	  --build-arg DOCKER_USER=$(shell whoami) \
          --load \
	  -t gl-client-js-builder \
	  ${JSDIR}

	sudo docker run -ti \
          -v ${REPO}/..:/io \
	  gl-client-js-builder \
	  bash -c "(cd public; make docker-build-js-linux)"

build-js: build-js-${OS}

package-js-linux: build-js-linux
	mkdir -p gl-client
	cp ${JSDIR}/index-${VERSION}-${ARCH}-unknown-linux-gnu.node gl-client/index.node
	mkdir -p ${JSDIR}/build/stage/${VERSION}/
	tar -cvzf ${PACKAGE} gl-client
	rm gl-client/index.node && rmdir gl-client

package-js-macos: build-js-macos
	mkdir -p gl-client
	cp ${JSDIR}/index-${VERSION}-${ARCH}-apple-darwin.node gl-client/index.node
	mkdir -p ${JSDIR}/build/stage/${VERSION}/
	tar -cvzf ${PACKAGE} gl-client
	rm gl-client/index.node && rmdir gl-client

# Packaging is a bit involved since we manually build the package
# structure and then package it into a platform specific
# distribution. But this level of access allows us to customize
# exactly what we ship.
package-js: package-js-${OS}

upload-bin-js: package-js
	gsutil cp -r ${JSDIR}/build/stage/${VERSION} gs://greenlight-artifacts/npm/gl-client-js

# Update the versions to match
${JSDIR}/package.json: ${JSDIR}/Cargo.toml
	jq ".version = \"${JSVERSION}\"" ${JSDIR}/package.json > ${JSDIR}/package.json.tmp
	mv ${JSDIR}/package.json.tmp ${JSDIR}/package.json

clean-js:
	cargo clean
	rm -rf ${JSDIR}/index.node ${JSDIR}/node_modules ${JSDIR}/build ${JSDIR}/proto.js

publish-js: upload-bin-js
	cd libs/gl-client-js/ && \
	npm publish || true  # Careful, there is no --skip-existing, so check the outcome here!

check-js: node_modules ${JSDIR}/index.node
	cd ${JSDIR}; npm install && npm run test

JSARTIFACTS = \
	${JSDIR}/index.node \
	${JSDIR}/build \
	${JSDIR}/index-${VERSION}-arm64-apple-darwin.node \
	${JSDIR}/index-${VERSION}-x86_64-apple-darwin.node \
	${JSDIR}/index-${VERSION}-${ARCH}-unknown-linux-gnu.node

ARTIFACTS += ${JSARTIFACTS}
