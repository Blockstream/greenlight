include libs/gl-client-py/Makefile
include libs/gl-client-js/Makefile

check: check-rs check-py check-js

check-rs:
	cd libs; cargo test --all -- --test-threads=1

clean-rs:
	cd libs; cargo clean

clean: clean-rs
