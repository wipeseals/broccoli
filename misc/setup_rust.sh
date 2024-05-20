#!/bin/bash -eu
set -o pipefail

rustup install nightly
rustup +nightly target install thumbv6m-none-eabi
cargo +nightly install flip-link
cargo +nightly install probe-rs --features=cli --locked
cargo +nightly install elf2uf2-rs --locked

