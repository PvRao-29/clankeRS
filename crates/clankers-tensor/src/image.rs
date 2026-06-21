use clankers_core::{RobotError, RobotResult};
use clankers_ros2::ImageMsg;
use ndarray::Array4;

use crate::layout::{DType, DataLayout};

/// Image tensor with robotics preprocessing helpers.
#[derive(Debug, Clone)]
pub struct ImageTensor {
    pub data: Array4<f32>,
    pub layout: DataLayout,
    pub dtype: DType,
    pub width: u32,
    pub height: u32,
}

impl ImageTensor {
    pub fn from_ros_msg(msg: &ImageMsg) -> RobotResult<Self> {
        let width = msg.width;
        let height = msg.height;
        if width == 0 || height == 0 {
            return Err(RobotError::Other("image has zero dimensions".into()));
        }

        let channels = if msg.encoding.contains("rgb") || msg.encoding.contains("bgr") {
            3
        } else if msg.encoding.contains("mono") || msg.encoding == "8UC1" {
            1
        } else {
            3
        };

        let expected = (width * height * channels) as usize;
        if msg.data.len() < expected {
            return Err(RobotError::Other(format!(
                "image data too short: expected {expected}, got {}",
                msg.data.len()
            )));
        }

        let mut hwc = Vec::with_capacity(expected);
        for &byte in msg.data.iter().take(expected) {
            hwc.push(byte as f32 / 255.0);
        }

        let channels = channels as usize;
        let array = Array4::from_shape_vec((1, height as usize, width as usize, channels), hwc)
            .map_err(|e| RobotError::Other(e.to_string()))?;

        Ok(Self {
            data: array,
            layout: DataLayout::Nhwc,
            dtype: DType::F32,
            width,
            height,
        })
    }

    pub fn resize(mut self, target_w: u32, target_h: u32) -> RobotResult<Self> {
        if target_w == 0 || target_h == 0 {
            return Err(RobotError::Other("invalid resize dimensions".into()));
        }
        // Nearest-neighbor resize on NHWC batch
        let (_n, h, w, c) = self.data.dim();
        let mut out = ndarray::Array4::<f32>::zeros((1, target_h as usize, target_w as usize, c));
        for y in 0..target_h as usize {
            for x in 0..target_w as usize {
                let src_y = y * h / target_h as usize;
                let src_x = x * w / target_w as usize;
                for ch in 0..c {
                    out[[0, y, x, ch]] = self.data[[0, src_y, src_x, ch]];
                }
            }
        }
        self.data = out;
        self.width = target_w;
        self.height = target_h;
        Ok(self)
    }

    pub fn normalize_imagenet(mut self) -> RobotResult<Self> {
        let mean = [0.485f32, 0.456, 0.406];
        let std = [0.229f32, 0.224, 0.225];
        let (_n, _h, _w, c) = self.data.dim();
        for pixel in self.data.iter_mut() {
            let _ = pixel;
        }
        if c >= 3 {
            for y in 0..self.data.dim().1 {
                for x in 0..self.data.dim().2 {
                    for ch in 0..3 {
                        self.data[[0, y, x, ch]] = (self.data[[0, y, x, ch]] - mean[ch]) / std[ch];
                    }
                }
            }
        }
        Ok(self)
    }

    pub fn to_nchw(mut self) -> RobotResult<Self> {
        if self.layout == DataLayout::Nchw {
            return Ok(self);
        }
        let (_n, h, w, c) = self.data.dim();
        let mut nchw = ndarray::Array4::<f32>::zeros((1, c, h, w));
        for y in 0..h {
            for x in 0..w {
                for ch in 0..c {
                    nchw[[0, ch, y, x]] = self.data[[0, y, x, ch]];
                }
            }
        }
        self.data = nchw;
        self.layout = DataLayout::Nchw;
        Ok(self)
    }

    pub fn to_vec(&self) -> Vec<f32> {
        self.data.iter().copied().collect()
    }

    pub fn shape(&self) -> Vec<usize> {
        self.data.shape().to_vec()
    }
}
