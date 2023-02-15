mod game_window_trait;
pub use game_window_trait::*;

#[cfg(feature="glfw_window")]
mod glfw_game_window;
#[cfg(feature="glfw_window")]
pub use glfw_game_window::*;

#[cfg(feature="glutin_window")]
pub use glutin_game_window::*;
#[cfg(feature="glutin_window")]
mod glutin_game_window;