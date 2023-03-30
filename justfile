# Some common commands for working on this stuff

test:
    cargo test --doc && cargo nextest run

chirp:
    cargo run --bin chirp -- tests/chip8-test-suite/bin/chip8-test-suite.ch8

cover:
    cargo llvm-cov --open --doctests

tokei:
    tokei --exclude tests/chip8-test-suite

