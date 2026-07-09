//! What a backend can and cannot do — negotiated by the engine at build time.

use clankers_tensor::{DType, Device};

/// A backend's self-reported capabilities.
///
/// The engine reads these to decide, per input, whether a borrowed view can be
/// bound directly (zero-copy) or must be materialised through the arena, and
/// whether `run_into` with preallocated outputs is possible.
#[derive(Debug, Clone)]
pub struct BackendCapabilities {
    /// Stable backend name (matches [`InferenceBackend::name`]).
    ///
    /// [`InferenceBackend::name`]: crate::backend::InferenceBackend::name
    pub name: &'static str,
    /// Whether the backend can read a borrowed, contiguous input without copying.
    pub zero_copy_inputs: bool,
    /// Whether the backend can write into caller-preallocated output buffers.
    pub supports_preallocated_outputs: bool,
    /// Dtypes the backend can consume.
    pub supported_dtypes: Vec<DType>,
    /// Devices the backend can run on.
    pub supported_devices: Vec<Device>,
}

impl BackendCapabilities {
    /// A conservative CPU-only, f32 capability set — a sensible default for
    /// backends to start from and adjust.
    pub fn cpu_f32(name: &'static str) -> Self {
        BackendCapabilities {
            name,
            zero_copy_inputs: false,
            supports_preallocated_outputs: false,
            supported_dtypes: vec![DType::F32],
            supported_devices: vec![Device::Cpu],
        }
    }

    /// Whether the backend accepts `dtype`.
    pub fn accepts_dtype(&self, dtype: DType) -> bool {
        self.supported_dtypes.contains(&dtype)
    }

    /// Whether the backend can run on `device`.
    pub fn accepts_device(&self, device: Device) -> bool {
        self.supported_devices.contains(&device)
    }
}
