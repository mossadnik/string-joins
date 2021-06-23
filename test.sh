#!/bin/bash
cargo test
cargo build --release

if [ ! -e generalized_suffix_array.so ]
then
    ln -s target/release/libgeneralized_suffix_array.so generalized_suffix_array.so
fi

python -m pytest tests/
