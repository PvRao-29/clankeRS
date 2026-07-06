#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayout {
    Hwc,
    Chw,
    Nhwc,
    Nchw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DType {
    U8,
    F16,
    F32,
    F64,
    I32,
    I64,
}
