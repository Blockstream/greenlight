
.PHONY: generate

GENERATED := \
	src/glclient.py \
	src/glclient.rb \
	src/glclient.swift \
	src/glclient/glclient.go \
	src/glclient/glclient.c \
	src/glclient/glclient.h \
	src/uniffi/glclient/glclient.kt

generate: src/glclient.py src/glclient.go src/glclient.rb src/glclient.swift src/glclient.kt

src/glclient.swift: src/glclient.udl src/glclient.uniffi.rs
	cargo run -p uniffi-bindgen --features=uniffi/cli generate src/glclient.udl --language swift --no-format

src/glclient.rb: src/glclient.udl src/glclient.uniffi.rs
	cargo run -p uniffi-bindgen --features=uniffi/cli generate src/glclient.udl --language ruby --no-format

src/uniffi/glclient/glclient.kt: src/glclient.udl src/glclient.uniffi.rs
	cargo run -p uniffi-bindgen --features=uniffi/cli generate src/glclient.udl --language kotlin --no-format

src/glclient.py: src/glclient.udl src/glclient.uniffi.rs
	cargo run -p uniffi-bindgen --features=uniffi/cli generate src/glclient.udl --language python --no-format

src/glclient/glclient.h src/glclient/glclient.c src/glclientglclient.go: src/glclient.udl src/glclient.uniffi.rs
	uniffi-bindgen-go src/glclient.udl --no-format

src/glclient.uniffi.rs: src/glclient.udl
	cargo run -p uniffi-bindgen --features=uniffi/cli scaffolding src/glclient.udl --no-format

clean:
	rm -fv ${GENERATED}
