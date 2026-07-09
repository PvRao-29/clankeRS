//! Type-erased inference runtime used by [`Model`](crate::Model).

use clankers_tensor::{TensorView, TensorViewMut};

use crate::backend::{NoopSession, TensorSpec};
use crate::inference::engine::InferenceEngine;
use crate::inference::named_outputs::NamedOutputs;
use crate::inference::{InferenceError, InferenceResult, InferenceStats};

#[cfg(feature = "onnxruntime")]
use crate::backend::OnnxSession;

/// The loaded, optimized inference runtime behind a [`Model`](crate::Model).
///
/// This is a type-erased wrapper around a backend-specific
/// [`InferenceEngine`](crate::inference::InferenceEngine). Most applications
/// should call methods on [`Model`](crate::Model) instead of using this type
/// directly.
#[derive(Debug)]
pub enum ModelEngine {
    #[cfg(feature = "onnxruntime")]
    Onnx(InferenceEngine<OnnxSession>),
    Noop(InferenceEngine<NoopSession>),
}

impl ModelEngine {
    pub fn backend_name(&self) -> &str {
        match self {
            #[cfg(feature = "onnxruntime")]
            Self::Onnx(e) => e.backend_name(),
            Self::Noop(e) => e.backend_name(),
        }
    }

    pub fn input_specs(&self) -> &[TensorSpec] {
        match self {
            #[cfg(feature = "onnxruntime")]
            Self::Onnx(e) => e.input_specs(),
            Self::Noop(e) => e.input_specs(),
        }
    }

    pub fn output_specs(&self) -> &[TensorSpec] {
        match self {
            #[cfg(feature = "onnxruntime")]
            Self::Onnx(e) => e.output_specs(),
            Self::Noop(e) => e.output_specs(),
        }
    }

    pub fn arena_bytes(&self) -> usize {
        match self {
            #[cfg(feature = "onnxruntime")]
            Self::Onnx(e) => e.arena_bytes(),
            Self::Noop(e) => e.arena_bytes(),
        }
    }

    pub fn run_named_with_stats<'a>(
        &mut self,
        named: &[(&'a str, TensorView<'a>)],
    ) -> InferenceResult<(NamedOutputs, InferenceStats)> {
        let (tensors, stats) = match self {
            #[cfg(feature = "onnxruntime")]
            Self::Onnx(e) => e.run_named_with_stats(named)?,
            Self::Noop(e) => e.run_named_with_stats(named)?,
        };
        let names: Vec<String> = self.output_specs().iter().map(|s| s.name.clone()).collect();
        Ok((NamedOutputs::from_specs_and_tensors(names, tensors), stats))
    }

    pub fn run_into<'a>(
        &mut self,
        named: &[(&'a str, TensorView<'a>)],
        outputs: &mut [TensorViewMut<'a>],
    ) -> InferenceResult<InferenceStats> {
        let ordered = self.order_named_inputs(named)?;
        match self {
            #[cfg(feature = "onnxruntime")]
            Self::Onnx(e) => e.run_into(&ordered, outputs),
            Self::Noop(e) => e.run_into(&ordered, outputs),
        }
    }

    fn order_named_inputs<'a>(
        &self,
        named: &[(&'a str, TensorView<'a>)],
    ) -> InferenceResult<Vec<TensorView<'a>>> {
        for (name, _) in named {
            if !self.input_specs().iter().any(|s| s.name == *name) {
                return Err(InferenceError::UnknownInput {
                    name: (*name).to_string(),
                    available: self
                        .input_specs()
                        .iter()
                        .map(|s| s.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", "),
                });
            }
        }
        let mut ordered = Vec::with_capacity(self.input_specs().len());
        for spec in self.input_specs() {
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
}

#[cfg(feature = "onnxruntime")]
impl From<InferenceEngine<OnnxSession>> for ModelEngine {
    fn from(engine: InferenceEngine<OnnxSession>) -> Self {
        ModelEngine::Onnx(engine)
    }
}

impl From<InferenceEngine<NoopSession>> for ModelEngine {
    fn from(engine: InferenceEngine<NoopSession>) -> Self {
        ModelEngine::Noop(engine)
    }
}
