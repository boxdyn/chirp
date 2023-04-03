# Some common commands for working on this stuff

# Run All Tests
rat:
    cargo test --doc && cargo nextest run

test:
    cargo nextest run

run rom:
    cargo run --bin chirp-minifb -- '{{rom}}'

debug rom:
    cargo run --bin chirp-minifb -- -d '{{rom}}'
# Run at 2100000 instructions per frame, and output per-frame runtime statistics
bench:
    cargo run --bin chirp-minifb --release -- chip8Archive/roms/1dcell.ch8 -Ps10 -S21000000 -m xochip

flame rom:
    CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -F 15300 --open --bin chirp-minifb -- '{{rom}}' -s10

flamebench:
    CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -F 15300 --open --bin chirp-minifb -- chip8Archive/roms/1dcell.ch8 -xPs10 -S2100000

cover:
    cargo llvm-cov --open --doctests

tokei:
    tokei --exclude chip8-test-suite --exclude chip8Archive

