default:
    @just --list

fix-rs:
    cargo clippy --fix --allow-dirty --allow-staged --all-targets --workspace -- -D warnings
fmt-rs:
    just fix-rs
    cargo fmt --all
fmt: fmt-rs

example id:
    cargo run --example {{ id }}

test:
    cargo test
t: test

# local ci ============================================================================================================

check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace --all-features --lib --tests
c: check
