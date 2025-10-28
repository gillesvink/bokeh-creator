#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

mod renderer;
mod settings;

// Re-exports for external use
pub use renderer::Renderer;
pub use settings::{FilterType, Noise, Settings};
