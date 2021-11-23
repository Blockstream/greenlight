ARCH=$(shell node -e "console.log(process.arch)")
PLAT=$(shell uname -s | tr '[:upper:]' '[:lower:]')
VERSION=$(shell jq '.version' package.json -r)
index.node:
	neon build --release

build: index.node

package: index.node
	npm install
	mkdir -p build/stage/${VERSION}/
	tar -cvzf build/stage/${VERSION}/gl-client-js-${VERSION}-${PLAT}-${ARCH}.tar.gz index.node

clean:
	cargo clean
	rm -f index.node
	rm -rf node_modules build

publish:
	rm index.node
	npx google-artifactregistry-auth
	npm publish
