//! Layout descriptors.
//!
//! Two distinct axes are modelled here and it is worth keeping them apart:
//!
//! * [`Layout`] describes the **memory** order of a tensor's elements — whether
//!   the byte buffer is a densely packed row-major block ([`Layout::Contiguous`])
//!   or carries explicit strides ([`Layout::Strided`]). This is what the zero-copy
//!   inference path reasons about.
//! * [`DataLayout`] describes the **semantic** dimension order of an image tensor
//!   (`NCHW` vs `NHWC`, …). A tensor can be `Contiguous` in memory and `Nchw`
//!   semantically at the same time.

// `DType` historically lived in this module; it now has its own file. Re-export
// it here so existing `use crate::layout::DType` paths keep compiling.
pub use crate::dtype::DType;

/// Memory layout of a tensor's element buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Layout {
    /// Row-major, densely packed: strides are implied by the shape and there is
    /// no padding between elements. This is the layout an ONNX runtime expects
    /// and the only one eligible for the zero-copy input path.
    #[default]
    Contiguous,
    /// Elements are laid out with explicit strides (e.g. a transposed or sliced
    /// view). Backends that require contiguous input must materialise a copy.
    Strided,
}

impl Layout {
    /// Whether this layout is densely packed row-major.
    pub const fn is_contiguous(self) -> bool {
        matches!(self, Layout::Contiguous)
    }
}

/// Semantic dimension ordering for image-like tensors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayout {
    /// Height, Width, Channels (single image, e.g. an OpenCV / ROS frame).
    Hwc,
    /// Channels, Height, Width (single image, PyTorch convention).
    Chw,
    /// Batch, Height, Width, Channels.
    Nhwc,
    /// Batch, Channels, Height, Width (the usual ONNX vision input).
    Nchw,
}

impl DataLayout {
    /// Parse a semantic layout from a config string (case-insensitive).
    pub fn parse(s: &str) -> Option<DataLayout> {
        match s.trim().to_ascii_uppercase().as_str() {
            "HWC" => Some(DataLayout::Hwc),
            "CHW" => Some(DataLayout::Chw),
            "NHWC" => Some(DataLayout::Nhwc),
            "NCHW" => Some(DataLayout::Nchw),
            _ => None,
        }
    }
}
