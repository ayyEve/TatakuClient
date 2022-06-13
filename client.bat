@echo off
mkdir game
cd game
set RUST_BACKTRACE=full
cargo run --release
cd ..
pause