default: watch

watch:
	cargo watch --clear --exec 'test --all'

test:
	cargo test --all

fmt:
	cargo +nightly fmt --all
