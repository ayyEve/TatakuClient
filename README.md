Tataku is a rhythme game with a focus on performance.
It started out as a taiko sim, but has since expanded to include popular games such as Osu and Mania (osu!mania, quaver, beatmania, etc)
Join our Discord server! https://discord.gg/PGa6XY7mKC

required deps:
 - windows:
   - cmake

 - linux (some may be incorrect, I'll double check when i have time)
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

 - build and run
  - cargo run --release
   

TODO:
- // Gameplay
 - letter ranking
 - multiplayer (oh boy lmao)
 - online replays

- // UI
 - skin folder overhaul (separate ui, gamemodes, etc)
 - make things not ugly (help ;-;)

- // Audio Engine
 - handle headphones being unplugged (might require a dropdown to select the output device)

- // Code
 - handle peppy direct download moment
 - depth doc (detail what is drawn at what depth range)

maybe todo:
 - profiler
 - more mods
