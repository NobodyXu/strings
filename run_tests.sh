#!/bin/bash -ex

cd $(dirname `realpath $0`)

export RUST_TEST_THREADS=1

rep=$(seq 1 10)

args="--all-features -- --nocapture"

for _ in $rep; do
    cargo test $args
done

export RUSTFLAGS='-Zsanitizer=address'
export RUSTDOCFLAGS="$RUSTFLAGS"
for _ in $rep; do
    cargo +nightly test $args
done

export MIRIFLAGS="-Zmiri-disable-isolation"
exec cargo +nightly miri test small_array_box $args
