.DEFAULT_GOAL := build

build:
	cargo build

test:
	RUST_LOG=debug cargo test

lint:
	cargo clippy

clean:
	cargo clean

update:  # Update dependencies listed in Cargo.lock
	cargo update  # fixes problems like this https://github.com/bodil/smartstring/issues/31

format:
	cargo fmt

run: build
	RUST_BACKTRACE=1 target/debug/client