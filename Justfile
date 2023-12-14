version := `cat ./crates/gdpm-cli/Cargo.toml | sed -n "s/^version = \"\(.*\)\"/\1/p"`

_default:
	@just -l

# Check code style
fmt:
	@cargo fmt --all

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

# Set crates version
set-version v:
	ls -d crates/gdpm-cli/Cargo.toml | xargs sed -i "s/^version = \"\(.*\)\"/version = \"{{ v }}\"/"

# Show version
show-version:
	@echo {{ version }}

# Install in path
install:
	cargo install --debug --path ./crates/gdpm-cli