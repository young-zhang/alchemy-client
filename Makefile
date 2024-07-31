.DEFAULT_GOAL := build

build:
	clear
	cargo build

test:
	clear
	RUST_BACKTRACE=1 RUST_LOG=debug cargo test

lint:
	clear
	cargo clippy

clean:
	cargo clean

update:  # Update dependencies listed in Cargo.lock
	cargo update  # fixes problems like this https://github.com/bodil/smartstring/issues/31

format:
	cargo fmt

run: build
	clear
	RUST_LOG=debug RUST_BACKTRACE=1 target/debug/client
