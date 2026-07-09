//! Tensor specifications: what a backend expects at each input/output slot.

use clankers_tensor::{DType, DataLayout, Layout, ShapeSpec, TensorView};

/// A description of one input or output tensor of a model.
///
/// Specs are reported by the backend (from ONNX metadata, or fixed for the
/// [`NoopBackend`](crate::backend::NoopBackend)) and are what the engine
/// validates caller-supplied tensors against.
#[derive(Debug, Clone)]
pub struct TensorSpec {
    /// The tensor's name, as the backend knows it (e.g. an ONNX input name).
    pub name: String,
    /// Required element type.
    pub dtype: DType,
    /// Required shape, with dynamic axes allowed.
    pub shape: ShapeSpec,
    /// Required memory layout. Backends today require [`Layout::Contiguous`].
    pub layout: Layout,
    /// Optional semantic dimension order, purely for human-readable messages.
    pub data_layout: Option<DataLayout>,
}

impl TensorSpec {
    /// A contiguous spec with no semantic-layout annotation.
    pub fn new(name: impl Into<String>, dtype: DType, shape: ShapeSpec) -> Self {
        TensorSpec {
            name: name.into(),
            dtype,
            shape,
            layout: Layout::Contiguous,
            data_layout: None,
        }
    }

    /// Attach a semantic layout tag (shown in error messages).
    pub fn with_data_layout(mut self, layout: DataLayout) -> Self {
        self.data_layout = Some(layout);
        self
    }

    /// A one-line description like `F32 [1,3,224,224] NCHW`.
    ///
    /// Dtype and semantic layout are upper-cased here so the message reads the
    /// way robotics engineers write shapes, distinct from the lower-case config
    /// spellings ([`DType::as_str`]).
    pub fn describe(&self) -> String {
        let dtype = self.dtype.as_str().to_uppercase();
        match self.data_layout {
            Some(dl) => format!(
                "{dtype} {} {}",
                self.shape,
                format!("{dl:?}").to_uppercase()
            ),
            None => format!("{dtype} {}", self.shape),
        }
    }

    /// Whether `view` satisfies this spec (dtype, shape, and contiguity).
    pub fn accepts(&self, view: &TensorView) -> bool {
        self.dtype == view.dtype()
            && self.shape.matches(view.shape())
            && layout_ok(self.layout, view)
    }

    /// Validate `view` against this spec, returning a `(expected, got)` pair on
    /// failure so the caller can build a rich [`InvalidInput`] error.
    ///
    /// [`InvalidInput`]: crate::inference::InferenceError::InvalidInput
    pub fn check<'a>(&self, view: &TensorView<'a>) -> Result<(), (String, String)> {
        if self.accepts(view) {
            Ok(())
        } else {
            Err((self.describe(), describe_view(view)))
        }
    }
}

fn layout_ok(required: Layout, view: &TensorView) -> bool {
    match required {
        Layout::Contiguous => view.is_contiguous(),
        Layout::Strided => true,
    }
}

/// Render a view for the "got" side of an error, e.g. `U8 [480,640,3]`.
pub fn describe_view(view: &TensorView) -> String {
    format!("{} {}", view.dtype().as_str().to_uppercase(), view.shape())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clankers_tensor::{DType, Shape};

    fn spec() -> TensorSpec {
        TensorSpec::new("state", DType::F32, ShapeSpec::from_onnx_dims(&[1, 12]))
    }

    #[test]
    fn accepts_matching_view() {
        let data = vec![0.0f32; 12];
        let shape = Shape::from([1, 12]);
        let view = TensorView::from_f32(&data, &shape).unwrap();
        assert!(spec().accepts(&view));
        assert!(spec().check(&view).is_ok());
    }

    #[test]
    fn rejects_dtype_and_reports_pair() {
        let data = vec![0u8; 12];
        let shape = Shape::from([1, 12]);
        let view = TensorView::from_slice(&data, DType::U8, &shape, Layout::Contiguous).unwrap();
        let (expected, got) = spec().check(&view).unwrap_err();
        assert_eq!(expected, "F32 [1,12]");
        assert_eq!(got, "U8 [1,12]");
    }

    #[test]
    fn describe_includes_semantic_layout() {
        let s = TensorSpec::new(
            "image",
            DType::F32,
            ShapeSpec::from_onnx_dims(&[1, 3, 224, 224]),
        )
        .with_data_layout(DataLayout::Nchw);
        assert_eq!(s.describe(), "F32 [1,3,224,224] NCHW");
    }
}
