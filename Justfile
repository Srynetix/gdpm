version := `cat ./crates/gdpm-cli/Cargo.toml | sed -n "s/^version = \"\(.*\)\"/\1/p"`

_default:
	@just -l

# Check code style
fmt:
	cargo fmt --all

# Check code style and error if changes are needed
fmt-check:
	cargo fmt --all -- --check

# Lint files
lint:
	cargo clippy --all-features --all --tests

# Lint files and error on warnings
lint-err:
	cargo clippy --all-features --all --tests -- -D warnings

# Debug build
build:
	cargo build --all

# Release build
build-release:
	cargo build --all --release

# Test
test:
	cargo test --all

# Test with coverage
test-cov:
	#!/usr/bin/env bash
	set -euo pipefail
	export CARGO_INCREMENTAL=0
	export RUSTFLAGS='-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'
	export RUSTDOCFLAGS='-Zprofile -Ccodegen-units=1 -Cinline-threshold=0 -Clink-dead-code -Coverflow-checks=off -Cpanic=abort -Zpanic_abort_tests'
	cargo test --all-features --no-fail-fast
	grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing --ignore "/*" --ignore "*/tests/*" -o ./target/debug/coverage/

# Set crates version
set-version v:
	ls -d crates/gdpm-*/Cargo.toml | xargs sed -i "s/^version = \"\(.*\)\"/version = \"{{ v }}\"/"


# Show version
show-version:
	@echo {{ version }}