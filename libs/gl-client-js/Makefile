ARCH=$(shell node -e "console.log(process.arch)")
PLAT=$(shell uname -s | tr '[:upper:]' '[:lower:]')
VERSION=$(shell jq '.version' package.json -r)

libs/gl-client-js/index.node: libs/gl-client-js/index.js
	cd libs/gl-client-js && npm install
	cd libs/gl-client-js && npm run build

# Packaging is a bit involved since we manually build the package
# structure and then package it into a platform specific
# distribution. But this level of access allows us to customize
# exactly what we ship.
package-js: libs/gl-client-js/index.node
	mkdir -p gl-client-js
	cp index.node gl-client-js
	mkdir -p build/stage/${VERSION}/
	tar -cvzf build/stage/${VERSION}/gl-client-js-${VERSION}-${PLAT}-${ARCH}.tar.gz gl-client-js
	rm gl-client-js/index.node
	rmdir gl-client-js

clean-js:
	rm -f libs/gl-client-js/index.node
	rm -rf libs/gl-client-js/node_modules libs/gl-client-js/build

publish-js:
	cd libs/gl-client-js/ && bash -x publish.sh

check-js: libs/gl-client-js/index.node
	cd libs/gl-client-js; npm run test
