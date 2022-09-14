REPO=$(shell git rev-parse --show-toplevel)
ARCH=$(shell node -e "console.log(process.arch)")
PLAT=$(shell uname -s | tr '[:upper:]' '[:lower:]')
VERSION=$(shell jq '.version' ${REPO}/libs/gl-client-js/package.json -r)
PACKAGE=${REPO}/libs/gl-client-js/build/stage/${VERSION}/gl-client-js-${VERSION}-${PLAT}-${ARCH}.tar.gz

index.node:
	npm run build

libs/gl-client-js/index.node: libs/gl-client-js/index.js
	cd libs/gl-client-js && npm install
	cd libs/gl-client-js && npm run build

# Packaging is a bit involved since we manually build the package
# structure and then package it into a platform specific
# distribution. But this level of access allows us to customize
# exactly what we ship.
package: index.node ${PACKAGE}
	npm run build
	mkdir -p gl-client-js
	cp index.node gl-client-js
	mkdir -p build/stage/${VERSION}/
	tar -cvzf ${PACKAGE} gl-client-js
	rm gl-client-js/index.node
	rmdir gl-client-js

upload: package
	gsutil cp -r build/stage/${VERSION} gs://greenlight-artifacts/npm/gl-client-js

clean-js:
	cargo clean
	rm -f index.node
	rm -rf node_modules build

publish-js:
	cd libs/gl-client-js/ && bash -x publish.sh

check-js: libs/gl-client-js/index.node
	cd libs/gl-client-js; npm run test
