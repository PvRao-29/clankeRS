//! Tensor shapes and shape specifications.

use smallvec::SmallVec;

/// Inline capacity for dimension vectors. Robotics tensors are almost always
/// rank ≤ 4 (`NCHW`) or a flat state vector, so six inline slots keep the common
/// case allocation-free.
pub type Dims = SmallVec<[usize; 6]>;

/// A concrete tensor shape: an ordered list of dimension sizes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shape {
    dims: Dims,
}

impl Shape {
    /// Build a shape from dimension sizes.
    pub fn new(dims: impl IntoIterator<Item = usize>) -> Self {
        Shape {
            dims: dims.into_iter().collect(),
        }
    }

    /// A rank-0 (scalar) shape.
    pub fn scalar() -> Self {
        Shape { dims: Dims::new() }
    }

    /// The dimension sizes.
    pub fn dims(&self) -> &[usize] {
        &self.dims
    }

    /// Number of dimensions.
    pub fn rank(&self) -> usize {
        self.dims.len()
    }

    /// Total number of elements (product of dimensions; `1` for a scalar).
    pub fn num_elements(&self) -> usize {
        self.dims.iter().product()
    }

    /// Number of bytes needed to store this shape densely for `element_size`.
    pub fn num_bytes(&self, element_size: usize) -> usize {
        self.num_elements() * element_size
    }

    /// Row-major (C-order) element strides for a contiguous buffer of this shape.
    ///
    /// The last dimension has stride `1`; each earlier stride is the product of
    /// all later dimension sizes.
    pub fn contiguous_strides(&self) -> Dims {
        let mut strides = Dims::with_capacity(self.dims.len());
        strides.resize(self.dims.len(), 1);
        let mut acc = 1usize;
        for i in (0..self.dims.len()).rev() {
            strides[i] = acc;
            acc *= self.dims[i];
        }
        strides
    }

    /// Whether `strides` describe a densely packed row-major layout of this shape.
    ///
    /// Dimensions of size `1` are ignored (their stride is irrelevant), matching
    /// how NumPy/ndarray treat unit axes.
    pub fn is_contiguous_with(&self, strides: &[usize]) -> bool {
        if strides.len() != self.dims.len() {
            return false;
        }
        let expected = self.contiguous_strides();
        self.dims
            .iter()
            .zip(strides.iter().zip(expected.iter()))
            .all(|(&d, (&got, &want))| d <= 1 || got == want)
    }
}

impl<const N: usize> From<[usize; N]> for Shape {
    fn from(dims: [usize; N]) -> Self {
        Shape {
            dims: SmallVec::from_slice(&dims),
        }
    }
}

impl From<&[usize]> for Shape {
    fn from(dims: &[usize]) -> Self {
        Shape {
            dims: SmallVec::from_slice(dims),
        }
    }
}

impl From<Vec<usize>> for Shape {
    fn from(dims: Vec<usize>) -> Self {
        Shape {
            dims: SmallVec::from_vec(dims),
        }
    }
}

impl std::fmt::Display for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, d) in self.dims.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{d}")?;
        }
        write!(f, "]")
    }
}

/// A single dimension in a [`ShapeSpec`]: either a fixed size or a dynamic axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dim {
    /// A fixed extent known at model-load time.
    Fixed(usize),
    /// A dynamic extent (ONNX `-1` / named symbolic dim), resolved at run time.
    Dynamic,
}

impl Dim {
    /// The fixed size, if this dimension is not dynamic.
    pub fn fixed(self) -> Option<usize> {
        match self {
            Dim::Fixed(n) => Some(n),
            Dim::Dynamic => None,
        }
    }

    /// Whether `size` is an acceptable concrete value for this dimension.
    pub fn accepts(self, size: usize) -> bool {
        match self {
            Dim::Fixed(n) => n == size,
            Dim::Dynamic => true,
        }
    }
}

impl std::fmt::Display for Dim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dim::Fixed(n) => write!(f, "{n}"),
            Dim::Dynamic => write!(f, "?"),
        }
    }
}

/// A shape template with optional dynamic axes, as reported by a backend for an
/// input or output tensor. Concrete [`Shape`]s are validated against it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ShapeSpec {
    dims: SmallVec<[Dim; 6]>,
}

impl ShapeSpec {
    /// Build a spec from a list of dimensions.
    pub fn new(dims: impl IntoIterator<Item = Dim>) -> Self {
        ShapeSpec {
            dims: dims.into_iter().collect(),
        }
    }

    /// Build a spec from raw ONNX-style extents where any negative value marks a
    /// dynamic axis.
    pub fn from_onnx_dims(dims: &[i64]) -> Self {
        ShapeSpec {
            dims: dims
                .iter()
                .map(|&d| {
                    if d < 0 {
                        Dim::Dynamic
                    } else {
                        Dim::Fixed(d as usize)
                    }
                })
                .collect(),
        }
    }

    /// The dimensions of this spec.
    pub fn dims(&self) -> &[Dim] {
        &self.dims
    }

    /// Number of dimensions.
    pub fn rank(&self) -> usize {
        self.dims.len()
    }

    /// Whether a concrete `shape` satisfies this spec (rank + every fixed axis).
    pub fn matches(&self, shape: &Shape) -> bool {
        self.dims.len() == shape.rank()
            && self
                .dims
                .iter()
                .zip(shape.dims())
                .all(|(dim, &size)| dim.accepts(size))
    }

    /// A representative concrete shape, substituting `1` for dynamic axes.
    ///
    /// Useful for warm-up buffers and diagnostics when no real input is present.
    pub fn concrete_or_unit(&self) -> Shape {
        Shape::new(self.dims.iter().map(|d| d.fixed().unwrap_or(1)))
    }
}

impl std::fmt::Display for ShapeSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, d) in self.dims.iter().enumerate() {
            if i > 0 {
                write!(f, ",")?;
            }
            write!(f, "{d}")?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn num_elements_and_strides() {
        let s = Shape::from([1, 3, 4, 4]);
        assert_eq!(s.rank(), 4);
        assert_eq!(s.num_elements(), 48);
        assert_eq!(s.contiguous_strides().as_slice(), &[48, 16, 4, 1]);
    }

    #[test]
    fn scalar_has_one_element() {
        let s = Shape::scalar();
        assert_eq!(s.rank(), 0);
        assert_eq!(s.num_elements(), 1);
        assert!(s.contiguous_strides().is_empty());
    }

    #[test]
    fn contiguity_check_ignores_unit_axes() {
        let s = Shape::from([1, 12]);
        assert!(s.is_contiguous_with(&[12, 1]));
        // Unit leading axis: its stride does not matter.
        assert!(s.is_contiguous_with(&[999, 1]));
        assert!(!s.is_contiguous_with(&[12, 2]));
        assert!(!s.is_contiguous_with(&[1])); // wrong rank
    }

    #[test]
    fn shape_from_array_slice_and_vec() {
        assert_eq!(Shape::from([2, 2]), Shape::from(vec![2, 2]));
        let slice: &[usize] = &[2, 2];
        assert_eq!(Shape::from(slice), Shape::from([2, 2]));
    }

    #[test]
    fn spec_matches_fixed_and_dynamic() {
        let spec = ShapeSpec::from_onnx_dims(&[-1, 3, 224, 224]);
        assert!(spec.matches(&Shape::from([1, 3, 224, 224])));
        assert!(spec.matches(&Shape::from([8, 3, 224, 224])));
        assert!(!spec.matches(&Shape::from([1, 1, 224, 224])));
        assert!(!spec.matches(&Shape::from([3, 224, 224])));
        assert_eq!(spec.concrete_or_unit(), Shape::from([1, 3, 224, 224]));
    }
}
