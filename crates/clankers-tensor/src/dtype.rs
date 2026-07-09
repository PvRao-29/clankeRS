//! Element data types for tensors.

/// The element type stored in a tensor's byte buffer.
///
/// Every variant maps to a fixed byte width via [`DType::element_size`], which
/// the view/owned tensor types use to validate that a byte buffer's length is
/// consistent with a [`Shape`](crate::Shape).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DType {
    /// 8-bit boolean (stored as a single byte, `0` or `1`).
    Bool,
    /// Unsigned 8-bit integer — the native pixel type for camera frames.
    U8,
    /// IEEE-754 half precision (16-bit). No native Rust type; treated as raw bytes.
    F16,
    /// IEEE-754 single precision. The default inference dtype.
    F32,
    /// IEEE-754 double precision.
    F64,
    /// Signed 32-bit integer.
    I32,
    /// Signed 64-bit integer (ONNX label/index tensors are commonly `i64`).
    I64,
}

impl DType {
    /// Size of a single element in bytes.
    pub const fn element_size(self) -> usize {
        match self {
            DType::Bool | DType::U8 => 1,
            DType::F16 => 2,
            DType::F32 | DType::I32 => 4,
            DType::F64 | DType::I64 => 8,
        }
    }

    /// Stable lowercase name, matching the strings used in `clankeRS.toml`.
    pub const fn as_str(self) -> &'static str {
        match self {
            DType::Bool => "bool",
            DType::U8 => "u8",
            DType::F16 => "f16",
            DType::F32 => "f32",
            DType::F64 => "f64",
            DType::I32 => "i32",
            DType::I64 => "i64",
        }
    }

    /// Parse a dtype from its [`DType::as_str`] name (case-insensitive).
    ///
    /// Accepts a few common aliases (`float32`, `float`, `int64`, …) so config
    /// files can use either spelling.
    pub fn parse(s: &str) -> Option<DType> {
        match s.trim().to_ascii_lowercase().as_str() {
            "bool" => Some(DType::Bool),
            "u8" | "uint8" => Some(DType::U8),
            "f16" | "float16" | "half" => Some(DType::F16),
            "f32" | "float32" | "float" => Some(DType::F32),
            "f64" | "float64" | "double" => Some(DType::F64),
            "i32" | "int32" | "int" => Some(DType::I32),
            "i64" | "int64" | "long" => Some(DType::I64),
            _ => None,
        }
    }
}

impl std::fmt::Display for DType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_sizes_are_correct() {
        assert_eq!(DType::Bool.element_size(), 1);
        assert_eq!(DType::U8.element_size(), 1);
        assert_eq!(DType::F16.element_size(), 2);
        assert_eq!(DType::F32.element_size(), 4);
        assert_eq!(DType::I32.element_size(), 4);
        assert_eq!(DType::F64.element_size(), 8);
        assert_eq!(DType::I64.element_size(), 8);
    }

    #[test]
    fn parse_roundtrips_and_aliases() {
        for dt in [
            DType::Bool,
            DType::U8,
            DType::F16,
            DType::F32,
            DType::F64,
            DType::I32,
            DType::I64,
        ] {
            assert_eq!(DType::parse(dt.as_str()), Some(dt));
        }
        assert_eq!(DType::parse("Float32"), Some(DType::F32));
        assert_eq!(DType::parse("int64"), Some(DType::I64));
        assert_eq!(DType::parse("nonsense"), None);
    }
}
