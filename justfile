default: watch

watch:
	cargo watch --clear --exec 'test --all'

test:
	cargo test --all

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all

publish: test clippy
	cargo publish
	git push github
