#pragma once

#include <array>
#include <cstddef>
#include <type_traits>

namespace clankers {
namespace detail {

/// Minimal C++17 `std::span` stand-in for borrowed buffers.
template <typename T>
class span {
public:
    using element_type = T;
    using value_type = std::remove_cv_t<T>;
    using size_type = std::size_t;

    constexpr span() noexcept : data_(nullptr), size_(0) {}

    constexpr span(T* data, size_type count) noexcept : data_(data), size_(count) {}

    template <std::size_t N>
    constexpr span(T (&arr)[N]) noexcept : data_(arr), size_(N) {}

    template <std::size_t N>
    constexpr span(std::array<std::remove_const_t<T>, N>& arr) noexcept
        : data_(arr.data()), size_(N) {}

    constexpr T* data() const noexcept { return data_; }
    constexpr size_type size() const noexcept { return size_; }
    constexpr bool empty() const noexcept { return size_ == 0; }

    constexpr T& operator[](size_type i) const { return data_[i]; }

    constexpr T* begin() const noexcept { return data_; }
    constexpr T* end() const noexcept { return data_ + size_; }

private:
    T* data_;
    size_type size_;
};

}  // namespace detail
}  // namespace clankers
