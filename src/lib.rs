#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

mod renderer;
mod datamodel {
    include!(concat!(env!("OUT_DIR"), "/bokeh_creator.data.rs"));
}

// Re-exports for external use
pub use crate::datamodel::*;
pub use renderer::Renderer;
