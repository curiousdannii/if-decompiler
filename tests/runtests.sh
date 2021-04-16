#!/bin/sh

set -e

cd "$(dirname "$0")"
RUST_BACKTRACE=1 cargo run --bin glulxtoc -- glulxercise.ulx --disassemble
mkdir -p glulxercise-cheapglk
cd glulxercise-cheapglk
cmake ../glulxercise.ulx.decompiled
make -j$(nproc) --no-print-directory