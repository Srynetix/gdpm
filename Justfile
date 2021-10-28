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