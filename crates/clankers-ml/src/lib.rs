//! ML inference and model deployment layer.

pub mod backend;
pub mod model;
pub mod validation;

pub use backend::ModelBackend;
pub use model::{Model, ModelBuilder, ModelMetadata};
pub use validation::{ModelValidator, ValidationReport};
