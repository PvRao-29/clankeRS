//! An owned, over-aligned byte buffer for tensor storage.

/// A heap byte buffer guaranteed to be 8-byte aligned.
///
/// The largest element we store is 8 bytes (`f64`/`i64`), so backing the buffer
/// with a `Vec<u64>` guarantees the start address satisfies the alignment of any
/// [`DType`](crate::DType). That in turn lets [`Tensor`](crate::Tensor) hand out
/// typed slices (`&[f32]`, …) by reinterpreting the bytes without a copy and
/// without risking an unaligned read.
///
/// Storing the words (rather than a raw `Vec<u8>`) also keeps `Drop` sound: the
/// allocation is always freed with the same `Vec<u64>` layout it was created
/// with.
#[derive(Debug, Clone)]
pub struct Buffer {
    /// 8-byte-aligned backing store, sized up to a whole number of `u64` words.
    words: Vec<u64>,
    /// Number of valid leading bytes (`<= words.len() * 8`).
    len: usize,
}

/// Round `bytes` up to a whole number of 8-byte words.
const fn words_for(bytes: usize) -> usize {
    bytes.div_ceil(8)
}

impl Buffer {
    /// A zero-initialised buffer of `len` bytes.
    pub fn zeroed(len: usize) -> Self {
        Buffer {
            words: vec![0u64; words_for(len)],
            len,
        }
    }

    /// Copy `bytes` into a fresh aligned buffer.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut buf = Buffer::zeroed(bytes.len());
        buf.as_mut_bytes().copy_from_slice(bytes);
        buf
    }

    /// Length in valid bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the buffer holds zero bytes.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The valid bytes as a shared slice. The pointer is 8-byte aligned.
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: `words` owns at least `len` bytes (rounded up to a word), the
        // pointer is valid for `len` bytes, and `u8` imposes no alignment. The
        // borrow is tied to `&self`.
        unsafe { std::slice::from_raw_parts(self.words.as_ptr() as *const u8, self.len) }
    }

    /// The valid bytes as a mutable slice. The pointer is 8-byte aligned.
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        // SAFETY: as `as_bytes`, plus exclusive access via `&mut self`.
        unsafe { std::slice::from_raw_parts_mut(self.words.as_mut_ptr() as *mut u8, self.len) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_eight_byte_aligned() {
        for len in [0usize, 1, 3, 7, 8, 9, 1000] {
            let buf = Buffer::zeroed(len);
            assert_eq!(buf.len(), len);
            assert_eq!(buf.as_bytes().as_ptr() as usize % 8, 0);
        }
    }

    #[test]
    fn roundtrips_bytes() {
        let src: Vec<u8> = (0..37).collect();
        let mut buf = Buffer::from_bytes(&src);
        assert_eq!(buf.as_bytes(), src.as_slice());
        buf.as_mut_bytes()[0] = 255;
        assert_eq!(buf.as_bytes()[0], 255);
    }
}
