default: watch

watch:
	cargo watch --clear --exec 'ltest --all'

test:
	cargo ltest --all

fmt:
	cargo fmt --all

clippy:
	cargo lclippy --all

publish: test clippy
	cargo publish
	git push github
