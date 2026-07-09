#pragma once

#include <clankers/clankers.h>
#include <clankers/detail/span.hpp>

#include <cstddef>
#include <cstdint>
#include <vector>

namespace clankers {

enum class DType {
    Bool = CLANKERS_D_TYPE_BOOL,
    U8 = CLANKERS_D_TYPE_U8,
    F16 = CLANKERS_D_TYPE_F16,
    F32 = CLANKERS_D_TYPE_F32,
    F64 = CLANKERS_D_TYPE_F64,
    I32 = CLANKERS_D_TYPE_I32,
    I64 = CLANKERS_D_TYPE_I64,
};

enum class Layout {
    Contiguous = CLANKERS_LAYOUT_CONTIGUOUS,
    Strided = CLANKERS_LAYOUT_STRIDED,
};

/// Concrete tensor shape (rank ≤ `CLANKERS_MAX_RANK`).
class Shape {
public:
    Shape() = default;
    explicit Shape(std::initializer_list<std::size_t> dims);

    std::size_t rank() const noexcept { return shape_.rank; }
    const std::size_t* dims() const noexcept { return shape_.dims; }
    ClankersShape c_shape() const noexcept { return shape_; }

    static Shape from_c(ClankersShape raw);

private:
    ClankersShape shape_{};
};

/// Borrowed read-only tensor view over caller-owned memory.
class TensorView {
public:
    static TensorView from_external(detail::span<const std::byte> data,
                                    DType dtype,
                                    Shape shape,
                                    Layout layout = Layout::Contiguous);

    const ClankersTensorView& c_view() const noexcept { return view_; }

private:
    ClankersTensorView view_{};
};

/// Borrowed writable tensor view over caller-owned memory.
class TensorViewMut {
public:
    static TensorViewMut from_external(detail::span<std::byte> data,
                                       DType dtype,
                                       Shape shape,
                                       Layout layout = Layout::Contiguous);

    ClankersTensorViewMut& c_view_mut() noexcept { return view_; }
    const ClankersTensorViewMut& c_view_mut() const noexcept { return view_; }

private:
    ClankersTensorViewMut view_{};
};

/// Owned tensor returned from inference (RAII).
class Tensor {
public:
    Tensor() = default;
    explicit Tensor(ClankersTensor* handle);
    ~Tensor();

    Tensor(const Tensor&) = delete;
    Tensor& operator=(const Tensor&) = delete;
    Tensor(Tensor&& other) noexcept;
    Tensor& operator=(Tensor&& other) noexcept;

    bool valid() const noexcept { return handle_ != nullptr; }
    detail::span<const std::byte> bytes() const;
    Shape shape() const;
    DType dtype() const;

private:
    ClankersTensor* handle_ = nullptr;
};

}  // namespace clankers
