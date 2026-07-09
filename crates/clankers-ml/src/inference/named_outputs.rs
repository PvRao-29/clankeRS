//! Named inference outputs keyed by ONNX graph output names.

use std::collections::HashMap;

use clankers_tensor::Tensor;

/// Outputs from a named inference run, keyed by each tensor's model output name.
#[derive(Debug, Clone, Default)]
pub struct NamedOutputs {
    order: Vec<String>,
    tensors: HashMap<String, Tensor>,
}

impl NamedOutputs {
    pub(crate) fn from_specs_and_tensors(
        names: impl IntoIterator<Item = String>,
        tensors: Vec<Tensor>,
    ) -> Self {
        let order: Vec<String> = names.into_iter().collect();
        let mut map = HashMap::with_capacity(order.len());
        for (name, tensor) in order.iter().cloned().zip(tensors) {
            map.insert(name, tensor);
        }
        NamedOutputs {
            order,
            tensors: map,
        }
    }

    /// Whether an output with this name was produced.
    pub fn contains(&self, name: &str) -> bool {
        self.tensors.contains_key(name)
    }

    /// Borrow one output tensor by name.
    pub fn get(&self, name: &str) -> Option<&Tensor> {
        self.tensors.get(name)
    }

    /// Output names in model order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.order.iter().map(String::as_str)
    }

    /// All outputs as `(name, tensor)` pairs in model order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Tensor)> {
        self.order
            .iter()
            .map(|n| (n.as_str(), self.tensors.get(n).expect("output map in sync")))
    }

    /// The first output tensor (convenience for single-output models).
    pub fn first(&self) -> Option<&Tensor> {
        self.order.first().and_then(|n| self.tensors.get(n))
    }
}
