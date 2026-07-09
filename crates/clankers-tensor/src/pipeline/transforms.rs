//! The [`Transform`] trait and a few concrete preprocessing steps.

use crate::error::{TensorError, TensorResult};
use crate::{DType, Tensor};

/// A preprocessing step: tensor in, tensor out.
///
/// Transforms are composed with [`then`](Transform::then) to build a pipeline.
/// They operate on owned [`Tensor`]s, so the output of one becomes the input of
/// the next with no aliasing concerns.
pub trait Transform {
    /// Apply this transform to `input`, producing a new tensor.
    fn apply(&self, input: Tensor) -> TensorResult<Tensor>;

    /// Chain `self` before `next`.
    fn then<T: Transform>(self, next: T) -> Chain<Self, T>
    where
        Self: Sized,
    {
        Chain {
            first: self,
            second: next,
        }
    }
}

/// Two transforms run in sequence (produced by [`Transform::then`]).
pub struct Chain<A, B> {
    first: A,
    second: B,
}

impl<A: Transform, B: Transform> Transform for Chain<A, B> {
    fn apply(&self, input: Tensor) -> TensorResult<Tensor> {
        self.second.apply(self.first.apply(input)?)
    }
}

/// Convert a tensor to `F32`, scaling each element by `scale`.
///
/// For raw `U8` camera pixels use [`ToF32::pixels`] (scale `1/255`); for an
/// already-`F32` tensor use [`ToF32::identity`].
pub struct ToF32 {
    /// Multiplier applied to every element after conversion to `f32`.
    pub scale: f32,
}

impl ToF32 {
    /// Convert `U8` pixels in `[0, 255]` to `f32` in `[0, 1]`.
    pub fn pixels() -> Self {
        ToF32 { scale: 1.0 / 255.0 }
    }

    /// Convert to `f32` without rescaling.
    pub fn identity() -> Self {
        ToF32 { scale: 1.0 }
    }
}

impl Transform for ToF32 {
    fn apply(&self, input: Tensor) -> TensorResult<Tensor> {
        let n = input.num_elements();
        let mut out = vec![0.0f32; n];
        match input.dtype() {
            DType::F32 => {
                for (o, &v) in out.iter_mut().zip(input.as_f32()?) {
                    *o = v * self.scale;
                }
            }
            DType::U8 => {
                for (o, &b) in out.iter_mut().zip(input.bytes()) {
                    *o = b as f32 * self.scale;
                }
            }
            other => {
                return Err(TensorError::Adapter(format!(
                    "ToF32 does not support input dtype {other}"
                )));
            }
        }
        Tensor::from_f32_vec(input.shape().clone(), out)
    }
}

/// Per-channel mean/standard-deviation normalisation over the **last** tensor
/// dimension (the channel axis of an HWC / NHWC image, or a feature axis).
///
/// `out[..c] = (in[..c] - mean[c]) / std[c]`, where `c` is the index within the
/// last dimension. Requires an `F32` input (chain [`ToF32`] first).
pub struct Normalize {
    /// Per-channel means, length equal to the channel count.
    pub mean: Vec<f32>,
    /// Per-channel standard deviations, same length as `mean`.
    pub std: Vec<f32>,
}

impl Normalize {
    /// A normaliser with explicit per-channel statistics.
    pub fn new(mean: Vec<f32>, std: Vec<f32>) -> Self {
        Normalize { mean, std }
    }

    /// The standard ImageNet RGB normalisation.
    pub fn imagenet() -> Self {
        Normalize {
            mean: vec![0.485, 0.456, 0.406],
            std: vec![0.229, 0.224, 0.225],
        }
    }
}

impl Transform for Normalize {
    fn apply(&self, input: Tensor) -> TensorResult<Tensor> {
        let channels = self.mean.len();
        if channels == 0 || self.std.len() != channels {
            return Err(TensorError::Adapter(format!(
                "normalize needs equal-length non-empty mean/std (got {} / {})",
                self.mean.len(),
                self.std.len()
            )));
        }
        let last = input.shape().dims().last().copied().unwrap_or(0);
        if last != channels {
            return Err(TensorError::Adapter(format!(
                "normalize expects last dim {channels}, tensor's is {last}"
            )));
        }

        let mut out = input.as_f32()?.to_vec();
        for (i, v) in out.iter_mut().enumerate() {
            let c = i % channels;
            *v = (*v - self.mean[c]) / self.std[c];
        }
        Tensor::from_f32_vec(input.shape().clone(), out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Shape;

    #[test]
    fn to_f32_scales_pixels() {
        let t = Tensor::from_bytes(DType::U8, Shape::from([2]), &[0, 255]).unwrap();
        let out = ToF32::pixels().apply(t).unwrap();
        assert_eq!(out.as_f32().unwrap(), &[0.0, 1.0]);
    }

    #[test]
    fn normalize_applies_per_channel() {
        // One RGB pixel = [1.0, 1.0, 1.0] normalised by imagenet stats.
        let t = Tensor::from_f32_vec(Shape::from([1, 1, 3]), vec![1.0, 1.0, 1.0]).unwrap();
        let out = Normalize::imagenet().apply(t).unwrap();
        let v = out.as_f32().unwrap();
        let expect = [
            (1.0 - 0.485) / 0.229,
            (1.0 - 0.456) / 0.224,
            (1.0 - 0.406) / 0.225,
        ];
        for (a, b) in v.iter().zip(expect.iter()) {
            assert!((a - b).abs() < 1e-6, "{a} vs {b}");
        }
    }

    #[test]
    fn chain_composes_pixels_then_normalize() {
        let pixel = Tensor::from_bytes(DType::U8, Shape::from([1, 1, 3]), &[255, 128, 0]).unwrap();
        let pipeline = ToF32::pixels().then(Normalize::imagenet());
        let out = pipeline.apply(pixel).unwrap();
        assert_eq!(out.dtype(), DType::F32);
        assert_eq!(out.shape(), &Shape::from([1, 1, 3]));
        // First channel: (255/255 - 0.485)/0.229.
        let expect0 = (1.0 - 0.485) / 0.229;
        assert!((out.as_f32().unwrap()[0] - expect0).abs() < 1e-6);
    }

    #[test]
    fn normalize_rejects_channel_mismatch() {
        let t = Tensor::from_f32_vec(Shape::from([1, 1, 4]), vec![0.0; 4]).unwrap();
        assert!(matches!(
            Normalize::imagenet().apply(t),
            Err(TensorError::Adapter(_))
        ));
    }
}
