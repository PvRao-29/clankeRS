//! IO binding: turning caller tensors into the [`BackendTensor`]s a
//! [`BackendSession`](crate::backend::BackendSession) consumes and produces.
//!
//! This is where the engine's zero-copy decision lives. [`bind_input`] compares a
//! caller's [`TensorView`] against the backend's [`TensorSpec`] and its
//! [`BackendCapabilities`]:
//!
//! * matches the spec and the backend can read a borrowed contiguous view →
//!   [`BackendTensor::Borrowed`], no copy;
//! * matches the spec but the backend needs to own its input memory → a
//!   contiguous copy materialised through the [`TensorArena`] (reusing a pooled
//!   buffer under [`AllocationPolicy::Preallocate`]), recorded as one conversion
//!   copy;
//! * doesn't match the spec → a structured [`InferenceError::InvalidInput`].
//!
//! [`AllocationPolicy::Preallocate`]: clankers_tensor::AllocationPolicy::Preallocate

use clankers_tensor::{TensorArena, TensorView, TensorViewMut};

use crate::backend::{BackendCapabilities, BackendTensor, TensorSpec};
use crate::inference::{InferenceError, InferenceStats};

/// Whether [`bind_input`] borrowed the caller's buffer or materialised a copy.
pub(crate) enum Binding<'a> {
    /// Bound directly by borrow — no arena buffer involved.
    Borrowed(BackendTensor<'a>),
    /// Materialised a conversion copy into an arena tensor checked out for `slot`.
    Converted(BackendTensor<'a>),
}

/// Adapt one caller input view to what the backend expects for slot `spec`,
/// recording any conversion copy in `stats` and checking out an arena buffer for
/// `slot` when a copy is required.
pub(crate) fn bind_input<'a>(
    slot: usize,
    view: &TensorView<'a>,
    spec: &TensorSpec,
    caps: &BackendCapabilities,
    arena: &mut TensorArena,
    stats: &mut InferenceStats,
) -> Result<Binding<'a>, InferenceError> {
    // Structural validation first: dtype, shape (with dynamic axes), and layout.
    if let Err((expected, got)) = spec.check(view) {
        return Err(InferenceError::InvalidInput {
            name: spec.name.clone(),
            expected,
            got,
        });
    }

    // The engine only runs on devices the backend advertises.
    if !caps.accepts_device(view.device()) {
        return Err(InferenceError::UnsupportedDevice(view.device().to_string()));
    }

    // Zero-copy path: the backend can read the caller's contiguous buffer directly.
    if caps.zero_copy_inputs && view.is_contiguous() {
        return Ok(Binding::Borrowed(BackendTensor::Borrowed(*view)));
    }

    // Fallback: the backend needs to own its input memory. Copy the (already
    // spec-conformant, contiguous) bytes into an arena buffer — reused across runs
    // under `Preallocate`. This is a real, counted copy.
    let mut owned = arena.checkout(slot, view.dtype(), view.shape().clone());
    owned.bytes_mut().copy_from_slice(view.bytes());
    stats.record_conversion_copy(view.num_bytes());
    Ok(Binding::Converted(BackendTensor::Owned(owned)))
}

/// Allocate a placeholder output tensor per output spec (used by `run`).
///
/// Each slot is sized from the spec (dynamic axes collapse to `1`). A backend
/// either fills its placeholder in place or replaces it with a freshly produced
/// [`BackendTensor::Owned`] — the norm for dynamically shaped outputs.
pub(crate) fn prepare_outputs(
    specs: &[TensorSpec],
    arena: &mut TensorArena,
) -> Vec<BackendTensor<'static>> {
    specs
        .iter()
        .map(|spec| BackendTensor::Owned(arena.alloc(spec.dtype, spec.shape.concrete_or_unit())))
        .collect()
}

/// Validate one caller-preallocated output buffer against its spec (used by
/// `run_into`), returning a structured [`InferenceError::InvalidOutput`] on
/// mismatch.
pub(crate) fn check_output(view: &TensorViewMut, spec: &TensorSpec) -> Result<(), InferenceError> {
    let ro = view.as_view();
    spec.check(&ro).map_err(|(expected, got)| InferenceError::InvalidOutput {
        name: spec.name.clone(),
        expected,
        got,
    })
}
