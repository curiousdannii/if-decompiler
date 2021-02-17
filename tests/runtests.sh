#!/bin/sh

cd "$(dirname "$0")"
cargo run --bin glulxtoc -- glulxercise.ulx
mkdir -p glulxercise-cheapglk
cd glulxercise-cheapglk
cmake ../glulxercise.ulx.decompiled
make -j$(nproc)