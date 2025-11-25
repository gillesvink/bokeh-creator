#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "renderer")]
mod renderer;
mod datamodel {
    include!(concat!(env!("OUT_DIR"), "/bokeh_creator.rs"));
}

// Re-exports for external use
pub use crate::datamodel::*;
#[cfg(feature = "renderer")]
pub use renderer::Renderer;
