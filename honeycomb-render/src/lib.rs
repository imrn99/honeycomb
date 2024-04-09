//! # honeycomb-render
//!
//! This crate implements a runner that can be used to display combinatorial maps.
//!
//! It currently only supports 2D maps as the core library only implements these (as [CMap2])
//!
//! ## Key bindings
//!
//! - Directional arrows -- Move up, down, left and right
//! - `F` -- Move forward (i.e. zoom in)
//! - `B` -- Move backward (i.e. zoom out)
//!
//! ## Quickstart
//!
//! Examples are available in the **honeycomb-examples** crate.

#[cfg(doc)]
use honeycomb_core::CMap2;

// ------ MODULE DECLARATIONS

mod camera;
mod handle;
mod runner;
mod shader_data;
mod state;

// ------ RE-EXPORTS

pub use handle::RenderParameters;
pub use runner::Runner;
pub use state::SmaaMode;
