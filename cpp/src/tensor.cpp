#include <clankers/error.hpp>
#include <clankers/tensor.hpp>

namespace clankers {

namespace {

ClankersDType to_c(DType dtype) { return static_cast<ClankersDType>(dtype); }

ClankersLayout to_c(Layout layout) { return static_cast<ClankersLayout>(layout); }

}  // namespace

Shape::Shape(std::initializer_list<std::size_t> dims) {
    shape_.rank = dims.size() > CLANKERS_MAX_RANK ? CLANKERS_MAX_RANK : dims.size();
    std::size_t i = 0;
    for (auto d : dims) {
        if (i >= CLANKERS_MAX_RANK) {
            break;
        }
        shape_.dims[i++] = d;
    }
}

Shape Shape::from_c(ClankersShape raw) {
    Shape s;
    s.shape_ = raw;
    return s;
}

TensorView TensorView::from_external(detail::span<const std::byte> data,
                                     DType dtype,
                                     Shape shape,
                                     Layout layout) {
    TensorView view;
    check(clankers_tensor_view_from_external(
        reinterpret_cast<const std::uint8_t*>(data.data()),
        data.size(),
        to_c(dtype),
        shape.c_shape(),
        to_c(layout),
        &view.view_));
    return view;
}

TensorViewMut TensorViewMut::from_external(detail::span<std::byte> data,
                                           DType dtype,
                                           Shape shape,
                                           Layout layout) {
    TensorViewMut view;
    check(clankers_tensor_view_mut_from_external(
        reinterpret_cast<std::uint8_t*>(data.data()),
        data.size(),
        to_c(dtype),
        shape.c_shape(),
        to_c(layout),
        &view.view_));
    return view;
}

Tensor::Tensor(ClankersTensor* handle) : handle_(handle) {}

Tensor::~Tensor() {
    if (handle_ != nullptr) {
        clankers_tensor_destroy(handle_);
        handle_ = nullptr;
    }
}

Tensor::Tensor(Tensor&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }

Tensor& Tensor::operator=(Tensor&& other) noexcept {
    if (this != &other) {
        if (handle_ != nullptr) {
            clankers_tensor_destroy(handle_);
        }
        handle_ = other.handle_;
        other.handle_ = nullptr;
    }
    return *this;
}

detail::span<const std::byte> Tensor::bytes() const {
    if (handle_ == nullptr) {
        return {};
    }
    const auto* data = clankers_tensor_data(handle_);
    const auto len = clankers_tensor_byte_len(handle_);
    return {reinterpret_cast<const std::byte*>(data), len};
}

Shape Tensor::shape() const {
    if (handle_ == nullptr) {
        return {};
    }
    return Shape::from_c(clankers_tensor_shape(handle_));
}

DType Tensor::dtype() const {
    if (handle_ == nullptr) {
        return DType::F32;
    }
    return static_cast<DType>(clankers_tensor_dtype(handle_));
}

}  // namespace clankers
