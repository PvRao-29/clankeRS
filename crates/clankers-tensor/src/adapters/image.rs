//! Borrow a ROS [`ImageMsg`] as a `U8` `HWC` tensor without copying its pixels.

use clankers_ros2::ImageMsg;

use crate::error::{TensorError, TensorResult};
use crate::{DType, Layout, Shape, TensorView};

/// The number of channels implied by a ROS image `encoding` string.
fn channels_for(encoding: &str) -> usize {
    let e = encoding.to_ascii_lowercase();
    if e.contains("rgba") || e.contains("bgra") {
        4
    } else if e.contains("rgb") || e.contains("bgr") {
        3
    } else if e.contains("mono") || e == "8uc1" {
        1
    } else {
        3
    }
}

/// A zero-copy borrow of an [`ImageMsg`] as a `U8` tensor of shape
/// `[height, width, channels]` (HWC).
///
/// Construction validates that the message's rows are tightly packed (no
/// `step` padding) so the borrowed byte slice is a genuinely contiguous tensor.
/// Padded frames must go through the copying [`ImageTensor`](crate::ImageTensor)
/// path instead.
pub struct ImageInput<'a> {
    data: &'a [u8],
    shape: Shape,
}

impl<'a> ImageInput<'a> {
    /// Borrow `msg`'s pixel buffer as an HWC tensor.
    pub fn from_msg(msg: &'a ImageMsg) -> TensorResult<Self> {
        if msg.width == 0 || msg.height == 0 {
            return Err(TensorError::Adapter("image has zero dimensions".into()));
        }
        let channels = channels_for(&msg.encoding);
        let row_bytes = msg.width as usize * channels;

        // A non-trivial `step` means padded rows, which are not contiguous.
        if msg.step != 0 && msg.step as usize != row_bytes {
            return Err(TensorError::Adapter(format!(
                "image row stride {} != {row_bytes} (padded rows are not zero-copy)",
                msg.step
            )));
        }

        let expected = msg.height as usize * row_bytes;
        if msg.data.len() < expected {
            return Err(TensorError::Adapter(format!(
                "image data too short: expected {expected} bytes, got {}",
                msg.data.len()
            )));
        }

        Ok(ImageInput {
            data: &msg.data[..expected],
            shape: Shape::from([msg.height as usize, msg.width as usize, channels]),
        })
    }

    /// The `[height, width, channels]` shape of the borrowed image.
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// Borrow the image as a read-only `U8` HWC [`TensorView`].
    pub fn view(&self) -> TensorView<'_> {
        TensorView::from_slice(self.data, DType::U8, &self.shape, Layout::Contiguous)
            .expect("image dimensions validated at construction")
    }
}

impl<'a> TryFrom<&'a ImageMsg> for ImageInput<'a> {
    type Error = TensorError;

    fn try_from(msg: &'a ImageMsg) -> TensorResult<Self> {
        ImageInput::from_msg(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb_msg(w: u32, h: u32) -> ImageMsg {
        let mut msg = ImageMsg::new(w, h, vec![0u8; (w * h * 3) as usize]);
        msg.encoding = "rgb8".into();
        // Fill with a recognisable ramp.
        for (i, b) in msg.data.iter_mut().enumerate() {
            *b = (i % 256) as u8;
        }
        msg
    }

    #[test]
    fn borrows_pixels_zero_copy() {
        let msg = rgb_msg(4, 2);
        let input = ImageInput::from_msg(&msg).unwrap();
        assert_eq!(input.shape(), &Shape::from([2, 4, 3]));
        let view = input.view();
        assert_eq!(view.dtype(), DType::U8);
        assert_eq!(view.num_elements(), 24);
        // The view points at the message's own buffer — no copy.
        assert_eq!(view.bytes().as_ptr(), msg.data.as_ptr());
    }

    #[test]
    fn mono_infers_single_channel() {
        let mut msg = ImageMsg::new(8, 8, vec![0u8; 64]);
        msg.encoding = "mono8".into();
        msg.step = 8; // one byte per pixel; `new` assumes rgb8's 3
        let input = ImageInput::from_msg(&msg).unwrap();
        assert_eq!(input.shape(), &Shape::from([8, 8, 1]));
    }

    #[test]
    fn rejects_padded_rows() {
        let mut msg = ImageMsg::new(4, 2, vec![0u8; 4 * 2 * 3]);
        msg.encoding = "rgb8".into();
        msg.step = 16; // padded (4*3 = 12)
        assert!(matches!(
            ImageInput::from_msg(&msg),
            Err(TensorError::Adapter(_))
        ));
    }

    #[test]
    fn rejects_short_data() {
        let mut msg = ImageMsg::new(4, 4, vec![0u8; 10]);
        msg.encoding = "rgb8".into();
        assert!(matches!(
            ImageInput::from_msg(&msg),
            Err(TensorError::Adapter(_))
        ));
    }

    #[test]
    fn try_from_works() {
        let msg = rgb_msg(2, 2);
        let input: ImageInput = (&msg).try_into().unwrap();
        assert_eq!(input.shape(), &Shape::from([2, 2, 3]));
    }
}
