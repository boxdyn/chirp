# Some common commands for working on this stuff

cover:
    cargo llvm-cov --open --doctests

test:
    cargo test --doc && cargo nextest run

tokei:
    tokei --exclude tests/chip8-test-suite
