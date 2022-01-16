@echo off
cargo build --release
cp C:\Users\Eve\Desktop\Projects\rust\taiko-rs\taiko.rs\target\release\taiko-rs-client.exe C:\Users\Eve\Desktop\taikors\taiko-rs-client.exe

start C:\Users\Eve\Desktop\Projects\rust\taiko-rs\taiko.rs\target\release\taiko-rs-client.exe
start C:\Users\Eve\Desktop\taikors\taiko-rs-client.exe