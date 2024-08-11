#!/bin/bash -eu
set -o pipefail

rustup toolchain add stable-2024-08-08
rustup target install thumbv6m-none-eabi
cargo install flip-link
cargo install probe-rs --features=cli --locked
cargo install elf2uf2-rs --locked

