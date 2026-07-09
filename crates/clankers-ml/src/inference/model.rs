//! Where a backend loads its model from.

use std::path::{Path, PathBuf};

/// The origin of a model to be loaded by an [`InferenceBackend`].
///
/// [`InferenceBackend`]: crate::backend::InferenceBackend
#[derive(Debug, Clone)]
pub enum ModelSource {
    /// A file on disk (e.g. an `.onnx` file).
    Path(PathBuf),
    /// Model bytes already in memory.
    Bytes(Vec<u8>),
    /// No external artifact — the backend is fully self-contained (e.g. the
    /// [`NoopBackend`](crate::backend::NoopBackend), which is defined by its specs).
    None,
}

impl ModelSource {
    /// A short human-readable description for diagnostics and error messages.
    pub fn describe(&self) -> String {
        match self {
            ModelSource::Path(p) => p.display().to_string(),
            ModelSource::Bytes(b) => format!("<{} in-memory bytes>", b.len()),
            ModelSource::None => "<none>".to_string(),
        }
    }

    /// The filesystem path, if this source is a path.
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            ModelSource::Path(p) => Some(p),
            _ => None,
        }
    }
}

impl From<PathBuf> for ModelSource {
    fn from(p: PathBuf) -> Self {
        ModelSource::Path(p)
    }
}

impl From<&Path> for ModelSource {
    fn from(p: &Path) -> Self {
        ModelSource::Path(p.to_path_buf())
    }
}

impl From<&str> for ModelSource {
    fn from(p: &str) -> Self {
        ModelSource::Path(PathBuf::from(p))
    }
}

impl From<Vec<u8>> for ModelSource {
    fn from(b: Vec<u8>) -> Self {
        ModelSource::Bytes(b)
    }
}
