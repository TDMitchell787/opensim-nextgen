#!/bin/bash
export RUST_LOG=info
export DYLD_LIBRARY_PATH=./zig/zig-out/lib
exec ../target/release/opensim-next "$@"
