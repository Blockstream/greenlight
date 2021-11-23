build:
	npm install

package:
	npm install
	npm run package

clean:
	cargo clean
	rm -f index.node
	rm -rf node_modules build

publish:
	rm index.node
	npx google-artifactregistry-auth
	npm publish
