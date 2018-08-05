all: test

test:
	@cargo test

check:
	@cargo +nightly clippy