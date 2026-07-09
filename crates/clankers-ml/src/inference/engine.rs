//! The [`InferenceEngine`]: orchestration over a loaded [`BackendSession`].
//!
//! **Most applications should use [`Model`](crate::Model) instead.** Construct an
//! `InferenceEngine` directly when implementing custom backends, custom allocation
//! policies, or advanced runtime integrations.

use std::time::Instant;

use clankers_tensor::{Device, Tensor, TensorArena, TensorView, TensorViewMut};

use crate::backend::{
    BackendCapabilities, BackendSession, BackendTensor, InferenceBackend, TensorSpec,
};
use crate::inference::builder::InferenceEngineBuilder;
use crate::inference::session::{bind_input, check_output, prepare_outputs, Binding};
use crate::inference::{InferenceError, InferenceResult, InferenceStats};

/// Lower-level inference runtime used by [`Model`](crate::Model).
///
/// Most applications should use [`Model`](crate::Model). Use `InferenceEngine` directly when
/// implementing custom backends, custom allocation policies, or advanced runtime
/// integrations.
pub struct InferenceEngine<S: BackendSession> {
    session: S,
    capabilities: BackendCapabilities,
    arena: TensorArena,
    backend_name: String,
    device: Device,
    /// Cached at build time so `run` never re-borrows the session for specs.
    input_specs: Vec<TensorSpec>,
    output_specs: Vec<TensorSpec>,
}

impl<S: BackendSession> std::fmt::Debug for InferenceEngine<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The session type need not be `Debug`; summarise the engine's shape instead.
        f.debug_struct("InferenceEngine")
            .field("backend", &self.backend_name)
            .field("device", &self.device)
            .field("inputs", &self.input_specs.len())
            .field("outputs", &self.output_specs.len())
            .field("arena_bytes", &self.arena.total_bytes_allocated())
            .finish_non_exhaustive()
    }
}

impl<S: BackendSession> InferenceEngine<S> {
    /// Start building an engine around `backend`.
    pub fn builder<B>(backend: B) -> InferenceEngineBuilder<B>
    where
        B: InferenceBackend<Session = S>,
    {
        InferenceEngineBuilder::new(backend)
    }

    /// Construct directly from an already-loaded session and its metadata. Most
    /// callers should go through [`InferenceEngine::builder`].
    pub(crate) fn from_parts(
        session: S,
        capabilities: BackendCapabilities,
        arena: TensorArena,
        backend_name: String,
        device: Device,
    ) -> Self {
        let input_specs = session.input_specs().to_vec();
        let output_specs = session.output_specs().to_vec();
        InferenceEngine {
            session,
            capabilities,
            arena,
            backend_name,
            device,
            input_specs,
            output_specs,
        }
    }

    /// The backend's name (`"noop"`, `"onnxruntime"`, …).
    pub fn backend_name(&self) -> &str {
        &self.backend_name
    }

    /// The device this engine runs on.
    pub fn device(&self) -> Device {
        self.device
    }

    /// The backend's negotiated capabilities.
    pub fn capabilities(&self) -> &BackendCapabilities {
        &self.capabilities
    }

    /// Specs for each model input, in order.
    pub fn input_specs(&self) -> &[TensorSpec] {
        &self.input_specs
    }

    /// Specs for each model output, in order.
    pub fn output_specs(&self) -> &[TensorSpec] {
        &self.output_specs
    }

    /// Total bytes the arena has allocated over the engine's lifetime.
    pub fn arena_bytes(&self) -> usize {
        self.arena.total_bytes_allocated()
    }

    /// Run inference over `inputs`, returning the produced output tensors.
    ///
    /// Inputs must be supplied in the same order as [`input_specs`](Self::input_specs).
    /// See [`run_with_stats`](Self::run_with_stats) for the accompanying
    /// [`InferenceStats`].
    pub fn run(&mut self, inputs: &[TensorView]) -> InferenceResult<Vec<Tensor>> {
        self.run_with_stats(inputs).map(|(outputs, _)| outputs)
    }

    /// Run inference over `inputs`, returning the outputs and per-run stats.
    pub fn run_with_stats(
        &mut self,
        inputs: &[TensorView],
    ) -> InferenceResult<(Vec<Tensor>, InferenceStats)> {
        if inputs.len() != self.input_specs.len() {
            return Err(InferenceError::InputCount {
                expected: self.input_specs.len(),
                got: inputs.len(),
            });
        }

        let mut stats = InferenceStats::default();
        let alloc_before = self.arena.total_allocations();
        let bytes_before = self.arena.total_bytes_allocated();

        let (bound, converted) = self.bind_all_inputs(inputs, &mut stats)?;
        // Placeholder outputs the backend fills or replaces.
        let mut outputs = prepare_outputs(&self.output_specs, &mut self.arena);

        let start = Instant::now();
        let backend_stats = self.session.run(&bound, &mut outputs)?;
        stats.latency = start.elapsed();

        self.checkin_conversions(bound, &converted);
        self.finalize_stats(&mut stats, backend_stats, alloc_before, bytes_before);

        let result = outputs.into_iter().map(|bt| bt.into_owned()).collect();
        Ok((result, stats))
    }

    /// Run inference writing outputs into caller-preallocated buffers.
    ///
    /// This is the robotics hot-loop path: when the inputs match the backend's
    /// spec and the `outputs` buffers match the output specs, no heap allocation
    /// occurs per call. Requires a backend that advertises
    /// [`supports_preallocated_outputs`](BackendCapabilities::supports_preallocated_outputs);
    /// otherwise returns a configuration error.
    pub fn run_into(
        &mut self,
        inputs: &[TensorView],
        outputs: &mut [TensorViewMut],
    ) -> InferenceResult<InferenceStats> {
        if !self.capabilities.supports_preallocated_outputs {
            return Err(InferenceError::UnsupportedPreallocatedOutputs {
                backend: self.backend_name.clone(),
            });
        }
        if inputs.len() != self.input_specs.len() {
            return Err(InferenceError::InputCount {
                expected: self.input_specs.len(),
                got: inputs.len(),
            });
        }
        if outputs.len() != self.output_specs.len() {
            return Err(InferenceError::Config(format!(
                "wrong number of output buffers: model produces {}, got {}",
                self.output_specs.len(),
                outputs.len()
            )));
        }
        // Every caller buffer must match its output spec before we bind it.
        for (buffer, spec) in outputs.iter().zip(self.output_specs.iter()) {
            check_output(buffer, spec)?;
        }

        let mut stats = InferenceStats::default();
        let alloc_before = self.arena.total_allocations();
        let bytes_before = self.arena.total_bytes_allocated();

        let (bound, converted) = self.bind_all_inputs(inputs, &mut stats)?;
        // Bind each caller buffer as a writable output the backend fills in place.
        let mut backend_outputs: Vec<BackendTensor> = outputs
            .iter_mut()
            .map(|buffer| BackendTensor::BorrowedMut(buffer.reborrow()))
            .collect();

        let start = Instant::now();
        let backend_stats = self.session.run(&bound, &mut backend_outputs)?;
        stats.latency = start.elapsed();

        // Release the caller-buffer reborrows before touching the arena again.
        drop(backend_outputs);
        self.checkin_conversions(bound, &converted);
        self.finalize_stats(&mut stats, backend_stats, alloc_before, bytes_before);
        Ok(stats)
    }

    /// Run inference with inputs supplied **by name**, in any order.
    ///
    /// The engine reorders them to match the backend's input specs, so callers
    /// don't have to track positional order for multi-input models:
    ///
    /// ```ignore
    /// engine.run_named(&[("image", image_view), ("state", state_view)])?;
    /// ```
    pub fn run_named(&mut self, named: &[(&str, TensorView)]) -> InferenceResult<Vec<Tensor>> {
        self.run_named_with_stats(named).map(|(outputs, _)| outputs)
    }

    /// [`run_named`](Self::run_named) with per-run [`InferenceStats`].
    pub fn run_named_with_stats(
        &mut self,
        named: &[(&str, TensorView)],
    ) -> InferenceResult<(Vec<Tensor>, InferenceStats)> {
        let ordered = self.order_named(named)?;
        self.run_with_stats(&ordered)
    }

    /// Reorder name-keyed inputs into the backend's expected positional order.
    fn order_named<'v>(
        &self,
        named: &[(&str, TensorView<'v>)],
    ) -> InferenceResult<Vec<TensorView<'v>>> {
        // Every supplied name must be a real model input.
        for (name, _) in named {
            if !self.input_specs.iter().any(|s| s.name == *name) {
                return Err(InferenceError::UnknownInput {
                    name: (*name).to_string(),
                    available: self
                        .input_specs
                        .iter()
                        .map(|s| s.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                });
            }
        }
        // Assemble inputs in the backend's declared order.
        let mut ordered = Vec::with_capacity(self.input_specs.len());
        for spec in &self.input_specs {
            match named.iter().find(|(n, _)| *n == spec.name).map(|(_, v)| *v) {
                Some(view) => ordered.push(view),
                None => {
                    return Err(InferenceError::InvalidInput {
                        name: spec.name.clone(),
                        expected: spec.describe(),
                        got: "<missing>".into(),
                    });
                }
            }
        }
        Ok(ordered)
    }

    /// Bind every input, returning the backend tensors and the slots whose
    /// conversion buffers must be checked back into the arena after the run.
    fn bind_all_inputs<'v>(
        &mut self,
        inputs: &[TensorView<'v>],
        stats: &mut InferenceStats,
    ) -> InferenceResult<(Vec<BackendTensor<'v>>, Vec<usize>)> {
        let mut bound = Vec::with_capacity(inputs.len());
        let mut converted = Vec::new();
        for (slot, (view, spec)) in inputs.iter().zip(self.input_specs.iter()).enumerate() {
            match bind_input(slot, view, spec, &self.capabilities, &mut self.arena, stats)? {
                Binding::Borrowed(bt) => bound.push(bt),
                Binding::Converted(bt) => {
                    converted.push(slot);
                    bound.push(bt);
                }
            }
        }
        Ok((bound, converted))
    }

    /// Return conversion buffers to the arena pool (a no-op under `Dynamic`).
    fn checkin_conversions(&mut self, bound: Vec<BackendTensor>, converted: &[usize]) {
        for (slot, bt) in bound.into_iter().enumerate() {
            if converted.contains(&slot) {
                if let BackendTensor::Owned(tensor) = bt {
                    self.arena.checkin(slot, tensor);
                }
            }
        }
    }

    /// Fold arena deltas and backend stats into the run's [`InferenceStats`].
    fn finalize_stats(
        &self,
        stats: &mut InferenceStats,
        backend: crate::backend::BackendRunStats,
        alloc_before: usize,
        bytes_before: usize,
    ) {
        stats.allocations = self.arena.total_allocations() - alloc_before;
        stats.bytes_allocated = self.arena.total_bytes_allocated() - bytes_before;
        stats.backend = backend;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clankers_tensor::{DType, Shape, ShapeSpec};

    use crate::backend::{
        BackendRunStats, BackendTensor, InferenceBackend, NoopBackend, TensorSpec,
    };
    use crate::inference::ModelSource;

    fn f32_view<'a>(data: &'a [f32], shape: &'a Shape) -> TensorView<'a> {
        TensorView::from_f32(data, shape).unwrap()
    }

    #[test]
    fn run_returns_identity_outputs() {
        // Milestone 3 deliverable: `engine.run(&[view])` yields outputs.
        let mut engine =
            InferenceEngine::builder(NoopBackend::identity(ShapeSpec::from_onnx_dims(&[1, 4])))
                .build()
                .unwrap();

        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let shape = Shape::from([1, 4]);
        let outputs = engine.run(&[f32_view(&data, &shape)]).unwrap();
        assert_eq!(outputs.len(), 1);
        assert_eq!(outputs[0].as_f32().unwrap(), data.as_slice());
    }

    #[test]
    fn zero_copy_path_reports_no_conversion_copies() {
        // NoopBackend advertises `zero_copy_inputs`, so a spec-matching contiguous
        // input is bound by borrow: engine conversion copies must be zero.
        let mut engine =
            InferenceEngine::builder(NoopBackend::identity(ShapeSpec::from_onnx_dims(&[1, 4])))
                .build()
                .unwrap();

        let data = vec![5.0f32; 4];
        let shape = Shape::from([1, 4]);
        let (_out, stats) = engine.run_with_stats(&[f32_view(&data, &shape)]).unwrap();
        assert_eq!(
            stats.clankers_copies, 0,
            "matching input must not be copied"
        );
        assert!(stats.is_zero_copy());
    }

    #[test]
    fn wrong_input_count_errors() {
        let mut engine =
            InferenceEngine::builder(NoopBackend::identity(ShapeSpec::from_onnx_dims(&[1, 4])))
                .build()
                .unwrap();
        let err = engine.run(&[]).unwrap_err();
        assert!(matches!(
            err,
            InferenceError::InputCount {
                expected: 1,
                got: 0
            }
        ));
    }

    #[test]
    fn dtype_mismatch_is_structured_error() {
        let mut engine =
            InferenceEngine::builder(NoopBackend::identity(ShapeSpec::from_onnx_dims(&[1, 4])))
                .build()
                .unwrap();

        // U8 input where the backend expects F32.
        let data = vec![0u8; 4];
        let shape = Shape::from([1, 4]);
        let view = TensorView::from_slice(
            &data,
            DType::U8,
            &shape,
            clankers_tensor::Layout::Contiguous,
        )
        .unwrap();
        let err = engine.run(&[view]).unwrap_err();
        match err {
            InferenceError::InvalidInput {
                name,
                expected,
                got,
            } => {
                assert_eq!(name, "input");
                assert_eq!(expected, "F32 [1,4]");
                assert_eq!(got, "U8 [1,4]");
            }
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    /// A backend that requires ownership of its inputs (cannot borrow), used to
    /// exercise the counted conversion-copy path.
    #[derive(Clone)]
    struct OwningBackend;

    struct OwningSession {
        inputs: Vec<TensorSpec>,
        outputs: Vec<TensorSpec>,
    }

    impl InferenceBackend for OwningBackend {
        type Session = OwningSession;
        fn name(&self) -> &'static str {
            "owning"
        }
        fn capabilities(&self) -> BackendCapabilities {
            let mut caps = BackendCapabilities::cpu_f32("owning");
            caps.zero_copy_inputs = false; // must own its inputs
            caps.supports_preallocated_outputs = false;
            caps
        }
        fn load_model(&self, _s: ModelSource) -> InferenceResult<OwningSession> {
            Ok(OwningSession {
                inputs: vec![TensorSpec::new(
                    "x",
                    DType::F32,
                    ShapeSpec::from_onnx_dims(&[1, 4]),
                )],
                outputs: vec![TensorSpec::new(
                    "y",
                    DType::F32,
                    ShapeSpec::from_onnx_dims(&[1, 4]),
                )],
            })
        }
    }

    impl BackendSession for OwningSession {
        fn input_specs(&self) -> &[TensorSpec] {
            &self.inputs
        }
        fn output_specs(&self) -> &[TensorSpec] {
            &self.outputs
        }
        fn run(
            &mut self,
            inputs: &[BackendTensor],
            outputs: &mut [BackendTensor],
        ) -> InferenceResult<BackendRunStats> {
            outputs[0] = BackendTensor::Owned(inputs[0].to_owned_tensor());
            Ok(BackendRunStats::ZERO)
        }
    }

    #[test]
    fn non_zero_copy_backend_records_a_conversion_copy() {
        let mut engine = InferenceEngine::builder(OwningBackend).build().unwrap();
        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let shape = Shape::from([1, 4]);
        let (out, stats) = engine.run_with_stats(&[f32_view(&data, &shape)]).unwrap();
        assert_eq!(
            stats.clankers_copies, 1,
            "owning backend must force one input copy"
        );
        assert_eq!(stats.clankers_bytes_copied, 16);
        assert!(stats.allocations >= 1);
        assert_eq!(out[0].as_f32().unwrap(), data.as_slice());
    }

    /// A backend that must own its inputs but can fill preallocated outputs — the
    /// combination that exercises both the conversion pool and `run_into`.
    #[derive(Clone)]
    struct InPlaceBackend;

    struct InPlaceSession {
        inputs: Vec<TensorSpec>,
        outputs: Vec<TensorSpec>,
    }

    impl InferenceBackend for InPlaceBackend {
        type Session = InPlaceSession;
        fn name(&self) -> &'static str {
            "inplace"
        }
        fn capabilities(&self) -> BackendCapabilities {
            let mut caps = BackendCapabilities::cpu_f32("inplace");
            caps.zero_copy_inputs = false; // must own its inputs
            caps.supports_preallocated_outputs = true; // can fill in place
            caps
        }
        fn load_model(&self, _s: ModelSource) -> InferenceResult<InPlaceSession> {
            let spec = |n| TensorSpec::new(n, DType::F32, ShapeSpec::from_onnx_dims(&[1, 4]));
            Ok(InPlaceSession {
                inputs: vec![spec("x")],
                outputs: vec![spec("y")],
            })
        }
    }

    impl BackendSession for InPlaceSession {
        fn input_specs(&self) -> &[TensorSpec] {
            &self.inputs
        }
        fn output_specs(&self) -> &[TensorSpec] {
            &self.outputs
        }
        fn run(
            &mut self,
            inputs: &[BackendTensor],
            outputs: &mut [BackendTensor],
        ) -> InferenceResult<BackendRunStats> {
            let mut stats = BackendRunStats::ZERO;
            for (input, slot) in inputs.iter().zip(outputs.iter_mut()) {
                let src = input.view();
                if let Some(dst) = slot.bytes_mut() {
                    dst.copy_from_slice(src.bytes());
                    stats.record_copy(src.num_bytes());
                } else {
                    *slot = BackendTensor::Owned(input.to_owned_tensor());
                }
            }
            Ok(stats)
        }
    }

    #[test]
    fn run_into_is_zero_alloc_in_a_hot_loop() {
        // Milestone 6 deliverable: a preallocated-output hot loop that never allocates.
        let mut engine =
            InferenceEngine::builder(NoopBackend::identity(ShapeSpec::from_onnx_dims(&[1, 4])))
                .allocation_policy(clankers_tensor::AllocationPolicy::Preallocate)
                .strict_realtime(true)
                .build()
                .unwrap();

        let input = vec![1.0f32, 2.0, 3.0, 4.0];
        let in_shape = Shape::from([1, 4]);
        let mut out = vec![0.0f32; 4];
        for i in 0..1000 {
            let view = f32_view(&input, &in_shape);
            let vm = TensorViewMut::from_f32(&mut out, Shape::from([1, 4])).unwrap();
            let stats = engine.run_into(&[view], &mut [vm]).unwrap();
            assert_eq!(stats.allocations, 0, "iteration {i} allocated");
            assert_eq!(
                stats.clankers_copies, 0,
                "iteration {i} copied a zero-copy input"
            );
        }
        assert_eq!(out, input);
    }

    #[test]
    fn strict_realtime_accepts_capable_backend() {
        assert!(
            InferenceEngine::builder(NoopBackend::identity(ShapeSpec::from_onnx_dims(&[1, 4])))
                .strict_realtime(true)
                .build()
                .is_ok()
        );
    }

    #[test]
    fn strict_realtime_rejects_incapable_backend() {
        let err = InferenceEngine::builder(OwningBackend)
            .strict_realtime(true)
            .build()
            .unwrap_err();
        assert!(matches!(err, InferenceError::RealtimeUnsatisfiable(_)));
    }

    #[test]
    fn run_into_requires_preallocated_output_support() {
        let mut engine = InferenceEngine::builder(OwningBackend).build().unwrap();
        let data = vec![0.0f32; 4];
        let shape = Shape::from([1, 4]);
        let mut out = vec![0.0f32; 4];
        let vm = TensorViewMut::from_f32(&mut out, Shape::from([1, 4])).unwrap();
        let err = engine
            .run_into(&[f32_view(&data, &shape)], &mut [vm])
            .unwrap_err();
        assert!(matches!(
            err,
            InferenceError::UnsupportedPreallocatedOutputs { .. }
        ));
    }

    #[test]
    fn run_into_validates_output_buffer() {
        let mut engine =
            InferenceEngine::builder(NoopBackend::identity(ShapeSpec::from_onnx_dims(&[1, 4])))
                .build()
                .unwrap();
        let data = vec![0.0f32; 4];
        let shape = Shape::from([1, 4]);
        let mut wrong = vec![0.0f32; 8]; // wrong length for a [1,4] output
        let vm = TensorViewMut::from_f32(&mut wrong, Shape::from([1, 8])).unwrap();
        let err = engine
            .run_into(&[f32_view(&data, &shape)], &mut [vm])
            .unwrap_err();
        assert!(matches!(err, InferenceError::InvalidOutput { .. }));
    }

    #[test]
    fn run_named_reorders_multi_input() {
        // A two-input identity: "state" then "image" in spec order.
        let specs = |n| TensorSpec::new(n, DType::F32, ShapeSpec::from_onnx_dims(&[1, 2]));
        let backend = NoopBackend::new(
            vec![specs("state"), specs("image")],
            vec![specs("state_out"), specs("image_out")],
        );
        let mut engine = InferenceEngine::builder(backend).build().unwrap();

        let state = vec![1.0f32, 2.0];
        let image = vec![3.0f32, 4.0];
        let shape = Shape::from([1, 2]);
        // Supply them in the *opposite* order to the specs.
        let outputs = engine
            .run_named(&[
                ("image", f32_view(&image, &shape)),
                ("state", f32_view(&state, &shape)),
            ])
            .unwrap();
        // Identity maps input i -> output i, so spec order is preserved.
        assert_eq!(outputs[0].as_f32().unwrap(), state.as_slice());
        assert_eq!(outputs[1].as_f32().unwrap(), image.as_slice());
    }

    #[test]
    fn run_named_rejects_unknown_input() {
        let mut engine =
            InferenceEngine::builder(NoopBackend::identity(ShapeSpec::from_onnx_dims(&[1, 4])))
                .build()
                .unwrap();
        let data = vec![0.0f32; 4];
        let shape = Shape::from([1, 4]);
        let err = engine
            .run_named(&[("not_a_real_input", f32_view(&data, &shape))])
            .unwrap_err();
        assert!(matches!(err, InferenceError::UnknownInput { .. }));
    }

    #[test]
    fn preallocate_reuses_conversion_buffer_after_warmup() {
        let mut engine = InferenceEngine::builder(InPlaceBackend)
            .allocation_policy(clankers_tensor::AllocationPolicy::Preallocate)
            .build()
            .unwrap();
        let data = vec![1.0f32, 2.0, 3.0, 4.0];
        let shape = Shape::from([1, 4]);
        let mut out = vec![0.0f32; 4];

        // First run converts the input (an allocation) and fills the output.
        let vm = TensorViewMut::from_f32(&mut out, Shape::from([1, 4])).unwrap();
        let s1 = engine
            .run_into(&[f32_view(&data, &shape)], &mut [vm])
            .unwrap();
        assert_eq!(
            s1.clankers_copies, 1,
            "owning backend forces an input conversion"
        );
        assert!(s1.allocations >= 1);

        // Second run reuses the pooled conversion buffer → no allocation.
        let vm = TensorViewMut::from_f32(&mut out, Shape::from([1, 4])).unwrap();
        let s2 = engine
            .run_into(&[f32_view(&data, &shape)], &mut [vm])
            .unwrap();
        assert_eq!(s2.clankers_copies, 1);
        assert_eq!(s2.allocations, 0, "conversion buffer should be reused");
        assert_eq!(out, data);
    }
}
