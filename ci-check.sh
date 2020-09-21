#!/bin/sh

set -ex

export PATH="$PATH":~/.cargo/bin
export RUST_BACKTRACE=1
export CARGO_INCREMENTAL=1

cargo build

# Allow some warnings on the very old compiler.
export RUSTFLAGS="-D warnings"

cargo test
cargo doc --no-deps

# Sometimes nightly doesn't have clippy or rustfmt, so don't try that there.
if [ "$TRAVIS_RUST_VERSION" = nightly ] ; then
	cargo test --benches
	cargo miri test miri
	exit
fi

cargo clippy --all --tests -- --deny clippy::all
cargo fmt --all -- --check
