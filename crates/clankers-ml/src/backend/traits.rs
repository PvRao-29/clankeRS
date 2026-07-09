//! The backend abstraction: a factory ([`InferenceBackend`]) that loads a model
//! into a runnable [`BackendSession`].

use crate::backend::{BackendCapabilities, BackendTensor, TensorSpec};
use crate::inference::{InferenceError, ModelSource};

/// A backend factory. One `InferenceBackend` can load many models, each into its
/// own [`BackendSession`]. Implementors are cheap, cloneable configuration
/// objects (a directory of options), not the loaded model itself.
pub trait InferenceBackend: Send + Sync {
    /// The session type this backend produces.
    type Session: BackendSession;

    /// A stable, human-readable name (`"noop"`, `"onnxruntime"`, …).
    fn name(&self) -> &'static str;

    /// What this backend can do — used by the engine to plan copies.
    fn capabilities(&self) -> BackendCapabilities;

    /// Load a model from `source` into a runnable session.
    fn load_model(&self, source: ModelSource) -> Result<Self::Session, InferenceError>;
}

/// A loaded model, ready to run. Sessions are stateful (`&mut self`) so backends
/// may keep IO binding scratch space between runs.
pub trait BackendSession: Send {
    /// Specs for each input, in order.
    fn input_specs(&self) -> &[TensorSpec];

    /// Specs for each output, in order.
    fn output_specs(&self) -> &[TensorSpec];

    /// Run inference.
    ///
    /// `inputs` are validated by the engine before this is called. Each element
    /// of `outputs` is either a preallocated buffer to fill in place, or a slot
    /// the backend replaces with a freshly produced [`BackendTensor::Owned`]
    /// (the norm for dynamically shaped outputs).
    fn run(
        &mut self,
        inputs: &[BackendTensor],
        outputs: &mut [BackendTensor],
    ) -> Result<BackendRunStats, InferenceError>;
}

/// Per-run accounting reported by the backend itself.
///
/// These are copies the *backend* performs internally (dtype packing, IO
/// binding, producing outputs). The engine tracks its own conversion copies
/// separately and sums the two into the public
/// [`InferenceStats`](crate::inference::InferenceStats).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BackendRunStats {
    /// Number of distinct copies the backend made this run.
    pub backend_copies: usize,
    /// Total bytes those copies moved.
    pub backend_bytes_copied: usize,
}

impl BackendRunStats {
    /// A run in which the backend copied nothing.
    pub const ZERO: BackendRunStats = BackendRunStats {
        backend_copies: 0,
        backend_bytes_copied: 0,
    };

    /// Record a single copy of `bytes` bytes.
    pub fn record_copy(&mut self, bytes: usize) {
        self.backend_copies += 1;
        self.backend_bytes_copied += bytes;
    }
}
