#!/bin/bash
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-unknown-linux-gnu-gcc \
cargo build --target=x86_64-unknown-linux-gnu --release