.PHONY: test
test:
	cargo test --lib
	cargo test --doc
	cargo test-sbf