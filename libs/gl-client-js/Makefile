ARCH=$(shell node -e "console.log(process.arch)")
PLAT=$(shell uname -s | tr '[:upper:]' '[:lower:]')
VERSION=$(shell jq '.version' package.json -r)
index.node:
	npm run build

build: index.node

# Packaging is a bit involved since we manually build the package
# structure and then package it into a platform specific
# distribution. But this level of access allows us to customize
# exactly what we ship.
package: index.node
	npm run build
	mkdir -p gl-client-js
	cp index.node gl-client-js
	mkdir -p build/stage/${VERSION}/
	tar -cvzf build/stage/${VERSION}/gl-client-js-${VERSION}-${PLAT}-${ARCH}.tar.gz gl-client-js
	rm gl-client-js/index.node
	rmdir gl-client-js

clean:
	cargo clean
	rm -f index.node
	rm -rf node_modules build

publish:
	bash -x publish.sh
