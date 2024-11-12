.PHONY: clippy
clippy:
	cargo clippy --no-deps --all-targets --all-features -- -D warnings
	cargo clippy -p sealevel-tools --no-deps --all-targets --no-default-features --features "noalloc-default" -- -D warnings

.PHONY: test
test:
	cargo test --lib
	cargo test --doc
	cargo test-sbf
