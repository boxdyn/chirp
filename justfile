# Some common commands for working on this stuff

# Run All Tests
rat:
    cargo test --doc && cargo nextest run

test:
    cargo nextest run

chirp:
    cargo run --bin chirp-minifb -- chip8-test-suite/bin/chip8-test-suite.ch8
# Run at 2100000 instructions per frame, and output per-frame runtime statistics
bench:
    cargo run --bin chirp-minifb --release -- chip8Archive/roms/1dcell.ch8 -xP -s10 -S2100000

flame rom:
    CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -F 15300 --open --bin chirp-minifb -- '{{rom}}' -s10

flamebench:
    CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -F 15300 --open --bin chirp-minifb -- chip8Archive/roms/1dcell.ch8 -xPs10 -S2100000

cover:
    cargo llvm-cov --open --doctests

tokei:
    tokei --exclude tests/chip8-test-suite

