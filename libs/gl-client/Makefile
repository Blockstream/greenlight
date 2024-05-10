check-rs:
	cd libs; cargo test --all -- --test-threads=1

clean-rs:
	cd libs; cargo clean

check-self-gl-client:
	(cd libs/gl-client; cargo test)

sync-files:
	rm -rf .resource/proto .resource/tls
	rsync -avr ../proto/ .resources/proto
	rsync -avr ../tls/ .resources/tls
	git add .resources
