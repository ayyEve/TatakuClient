@echo off
mkdir game
cd game
set RUST_BACKTRACE=1
cargo run --release
cd ..