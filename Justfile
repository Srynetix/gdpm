version := `cat ./crates/gdpm/Cargo.toml | sed -n "s/^version = \"\(.*\)\"/\1/p"`

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

# Run
run *ARGS:
	cargo run -p gdpm -- {{ARGS}}

# Set crates version
set-version v:
	ls -d crates/gdpm/Cargo.toml | xargs sed -i "s/^version = \"\(.*\)\"/version = \"{{ v }}\"/"

# Show version
show-version:
	@echo {{ version }}

# Install in path
install:
	cargo install --debug --path ./crates/gdpm

test-cov:
	cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info