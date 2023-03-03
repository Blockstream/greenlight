REPO=$(shell git rev-parse --show-toplevel)
ARCH=$(shell node -e "console.log(process.arch)")
PLAT=$(shell uname -s | tr '[:upper:]' '[:lower:]')
JSDIR=${REPO}/libs/gl-client-js
VERSION=$(shell jq '.version' ${JSDIR}/package.json -r)
JSVERSION=$(shell cd libs && cargo metadata --format-version 1 | jq -r '.packages[] | select(.name == "gl-client-js") | .version')
JSSRC=${JSDIR}/index.js ${JSDIR}/index.test.js
JSRSSRC=$(shell find ${JSDIR} -name '*.rs')

PACKAGE=${JSDIR}/build/stage/${VERSION}/gl-client-js-${VERSION}-${PLAT}-${ARCH}.tar.gz

.PHONY: build-js-macos-${ARCH} build-js-macos build-js-linux build-js
.NOTPARALLEL: build-js-macos-arm64 build-js-macos-x86_64 build-js-linux

#${JSDIR}/index.node: ${JSSRC} ${JSRSSRC}
#	cd libs/gl-client-js && npm install jest cargo-cp-artifact @mapbox/node-pre-gyp rewire
#	cd libs/gl-client-js && npm run build
#	cd ${JSDIR}; mv index.node index-${ARCH}-



build-js-macos-${ARCH}: ${JSSRC} ${JSRSSRC} 
	cd ${JSDIR} && \
	TARGET=${RSARCH}-apple-darwin npm run build && \
	mv index.node index-${VERSION}-${ARCH}-apple-darwin.node

build-js-macos: build-js-macos-${ARCH}

build-js-linux: ${JSSRC} ${JSRSSRC}
	echo "Building linux"
	cd ${JSDIR} && \
	TARGET=${ARCH}-unknown-linux-gnu npm run build && \
	mv index.node index-${VERSION}-${ARCH}-unknown-linux-gnu.node

build-js: build-js-${OS}

package-js-linux: build-js-linux
	mkdir -p gl-client-js
	cp ${JSDIR}/index-${VERSION}-${ARCH}-unknown-linux-gnu.node gl-client-js/index.node
	mkdir -p ${JSDIR}/build/stage/${VERSION}/
	tar -cvzf ${PACKAGE} gl-client-js
	rm gl-client-js/index.node && rmdir gl-client-js

package-js-macos: build-js-macos
	mkdir -p gl-client-js
	cp ${JSDIR}/index-${VERSION}-${RSARCH}-apple-darwin.node gl-client-js/index.node
	mkdir -p ${JSDIR}/build/stage/${VERSION}/
	tar -cvzf ${JSDIR}/build/stage/${VERSION}/gl-client-js-${VERSION}-${PLAT}-${ARCH}.tar.gz gl-client-js
	rm gl-client-js/index.node && rmdir gl-client-js

# Packaging is a bit involved since we manually build the package
# structure and then package it into a platform specific
# distribution. But this level of access allows us to customize
# exactly what we ship.
package-js: package-js-${OS}
	mkdir -p gl-client-js

upload-bin-js: package-js
	gsutil cp -r ${JSDIR}/build/stage/${VERSION} gs://greenlight-artifacts/npm/gl-client-js

# Update the versions to match
${JSDIR}/package.json: ${JSDIR}/Cargo.toml
	jq ".version = \"${JSVERSION}\"" ${JSDIR}/package.json > ${JSDIR}/package.json.tmp
	mv ${JSDIR}/package.json.tmp ${JSDIR}/package.json

clean-js:
	cargo clean
	rm -rf ${JSDIR}/index.node ${JSDIR}/node_modules ${JSDIR}/build

publish-js: upload-bin-js
	cd libs/gl-client-js/ && \
	npm run artifactregistry-login && \
	npm publish || true  # Careful, there is no --skip-existing, so check the outcome here!


check-js: ${JSDIR}/index.node
	cd ${JSDIR}; npm install && npm run test

JSARTIFACTS = \
	${JSDIR}/index.node \
	${JSDIR}/build \
	${JSDIR}/index-${VERSION}-arm64-apple-darwin.node \
	${JSDIR}/index-${VERSION}-x86_64-apple-darwin.node \
	${JSDIR}/index-${VERSION}-${ARCH}-unknown-linux-gnu.node

ARTIFACTS += ${JSARTIFACTS}
