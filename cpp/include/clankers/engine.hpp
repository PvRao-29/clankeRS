#pragma once

#include <clankers/clankers.h>
#include <clankers/detail/span.hpp>
#include <clankers/stats.hpp>
#include <clankers/tensor.hpp>

#include <memory>
#include <string>
#include <vector>

namespace clankers {

struct TensorSpec {
    std::string name;
    DType dtype = DType::F32;
    Shape shape;
    Layout layout = Layout::Contiguous;
};

/// Inference engine (RAII over `ClankersEngine`).
class Engine {
public:
    class Builder {
    public:
        Builder();
        ~Builder();

        Builder(const Builder&) = delete;
        Builder& operator=(const Builder&) = delete;
        Builder(Builder&&) noexcept;
        Builder& operator=(Builder&&) noexcept;

        Builder& model_path(const char* path);
        Builder& backend(const char* name);
        Builder& noop_shape(detail::span<const std::size_t> dims);
        Builder& warmup(std::uint32_t runs);
        Builder& strict_realtime(bool enabled);

        Engine build();

    private:
        ClankersEngineBuilder* builder_ = nullptr;
    };

    Engine();
    explicit Engine(ClankersEngine* handle);
    ~Engine();

    Engine(const Engine&) = delete;
    Engine& operator=(const Engine&) = delete;
    Engine(Engine&& other) noexcept;
    Engine& operator=(Engine&& other) noexcept;

    static Builder builder();

    std::size_t input_count() const;
    std::size_t output_count() const;
    TensorSpec input_spec(std::size_t index) const;
    TensorSpec output_spec(std::size_t index) const;

    std::vector<Tensor> run(detail::span<const TensorView> inputs);
    std::vector<Tensor> run_with_stats(detail::span<const TensorView> inputs,
                                       InferenceStats& stats);
    InferenceStats run_into(detail::span<const TensorView> inputs,
                            detail::span<TensorViewMut> outputs);

    ClankersEngine* handle() noexcept { return handle_; }

private:
    ClankersEngine* handle_ = nullptr;
};

}  // namespace clankers
