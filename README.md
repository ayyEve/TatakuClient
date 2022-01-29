My taiko sim made in rust. why? idk. enjoy!
Join our Discord server! https://discord.gg/PGa6XY7mKC

required deps:
 - windows:
   - cmake

 - linux (some may be incorrect, i'll double check when i have time)
   - gcc
   - cmake
   - libasound2-dev
   - pkg-config
   - libssl-dev
   - xorg-dev
   - libxcb-shape0
   - libxcb-render0
   - libxcb-fixes0

How to build:
 - install rust (https://rustup.rs/)
 - add nightly toolchain (required until iter_mut is added to stable)
   - rustup toolchain add nightly
   - rustup override set nightly

 - switch to animations branch (optional but preferred)
   - git checkout animations

 - build and run
  - cargo run --release
   

TODO:
- // UI
 - dropdown menu item
  
- // Gameplay
 - letter ranking
 - multiplayer (oh boy lmao)
 - online replays
 - skin support (need better slider rendering and image coloring first)

- // New Audio Engine
 - handle headphones being unplugged (might require a dropdown to select the output device)

- // Code
 - handle peppy direct download moment
 - pass the whole keys list instead of one key at a time
 - depth doc (detail what is drawn at what depth range)
  
maybe todo:
 - profiler
 - read osu replays
 - more mods