//! Zero-copy adapters from sensor messages and plain buffers into tensor views.
//!
//! Each adapter owns the small [`Shape`](crate::Shape) it derives (inline, no
//! heap allocation for typical ranks) and *borrows* the sensor payload, lending a
//! [`TensorView`](crate::TensorView) via `view()`. Holding the adapter keeps the
//! shape alive for as long as the borrowed view is needed — the pattern the
//! inference engine's `run` / `run_named` expect.

pub mod image;
pub mod state;

pub use image::ImageInput;
pub use state::StateInput;
