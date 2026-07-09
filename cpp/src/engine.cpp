#include <clankers/engine.hpp>
#include <clankers/error.hpp>

#include <utility>

namespace clankers {

namespace {

TensorSpec spec_from_c(ClankersTensorSpec raw) {
    TensorSpec spec;
    if (raw.name != nullptr) {
        spec.name = raw.name;
    }
    spec.dtype = static_cast<DType>(raw.dtype);
    spec.shape = Shape::from_c(raw.shape);
    spec.layout = static_cast<Layout>(raw.layout);
    clankers_tensor_spec_destroy(&raw);
    return spec;
}

}  // namespace

Engine::Builder::Builder() : builder_(clankers_engine_builder_new()) {
    if (builder_ == nullptr) {
        throw Error(CLANKERS_STATUS_ALLOCATION, "failed to create engine builder");
    }
}

Engine::Builder::~Builder() {
    if (builder_ != nullptr) {
        clankers_engine_builder_destroy(builder_);
        builder_ = nullptr;
    }
}

Engine::Builder::Builder(Builder&& other) noexcept : builder_(other.builder_) {
    other.builder_ = nullptr;
}

Engine::Builder& Engine::Builder::operator=(Builder&& other) noexcept {
    if (this != &other) {
        if (builder_ != nullptr) {
            clankers_engine_builder_destroy(builder_);
        }
        builder_ = other.builder_;
        other.builder_ = nullptr;
    }
    return *this;
}

Engine::Builder& Engine::Builder::model_path(const char* path) {
    check(clankers_engine_builder_set_model_path(builder_, path));
    return *this;
}

Engine::Builder& Engine::Builder::backend(const char* name) {
    check(clankers_engine_builder_set_backend(builder_, name));
    return *this;
}

Engine::Builder& Engine::Builder::noop_shape(detail::span<const std::size_t> dims) {
    check(clankers_engine_builder_set_noop_shape(builder_, dims.data(), dims.size()));
    return *this;
}

Engine::Builder& Engine::Builder::warmup(std::uint32_t runs) {
    check(clankers_engine_builder_set_warmup(builder_, runs));
    return *this;
}

Engine::Builder& Engine::Builder::strict_realtime(bool enabled) {
    check(clankers_engine_builder_set_strict_realtime(builder_, enabled));
    return *this;
}

Engine Engine::Builder::build() {
    ClankersEngine* engine = nullptr;
    check(clankers_engine_builder_build(builder_, &engine));
    builder_ = nullptr;
    return Engine(engine);
}

Engine::Engine() = default;

Engine::Engine(ClankersEngine* handle) : handle_(handle) {}

Engine::~Engine() {
    if (handle_ != nullptr) {
        clankers_engine_destroy(handle_);
        handle_ = nullptr;
    }
}

Engine::Engine(Engine&& other) noexcept : handle_(other.handle_) { other.handle_ = nullptr; }

Engine& Engine::operator=(Engine&& other) noexcept {
    if (this != &other) {
        if (handle_ != nullptr) {
            clankers_engine_destroy(handle_);
        }
        handle_ = other.handle_;
        other.handle_ = nullptr;
    }
    return *this;
}

Engine::Builder Engine::builder() { return Builder{}; }

std::size_t Engine::input_count() const { return clankers_engine_input_count(handle_); }

std::size_t Engine::output_count() const { return clankers_engine_output_count(handle_); }

TensorSpec Engine::input_spec(std::size_t index) const {
    ClankersTensorSpec raw{};
    check(clankers_engine_input_spec(handle_, index, &raw));
    return spec_from_c(raw);
}

TensorSpec Engine::output_spec(std::size_t index) const {
    ClankersTensorSpec raw{};
    check(clankers_engine_output_spec(handle_, index, &raw));
    return spec_from_c(raw);
}

std::vector<Tensor> Engine::run(detail::span<const TensorView> inputs) {
    InferenceStats ignored;
    return run_with_stats(inputs, ignored);
}

std::vector<Tensor> Engine::run_with_stats(detail::span<const TensorView> inputs,
                                           InferenceStats& stats) {
    std::vector<ClankersTensorView> c_inputs;
    c_inputs.reserve(inputs.size());
    for (const auto& view : inputs) {
        c_inputs.push_back(view.c_view());
    }

    ClankersTensor** outputs = nullptr;
    std::size_t count = 0;
    ClankersInferenceStats raw_stats{};
    check(clankers_engine_run_with_stats(
        handle_,
        c_inputs.empty() ? nullptr : c_inputs.data(),
        c_inputs.size(),
        &outputs,
        &count,
        &raw_stats));
    stats = InferenceStats::from_c(raw_stats);

    std::vector<Tensor> result;
    result.reserve(count);
    for (std::size_t i = 0; i < count; ++i) {
        result.emplace_back(outputs[i]);
    }
    clankers_output_array_destroy(outputs, count);
    return result;
}

InferenceStats Engine::run_into(detail::span<const TensorView> inputs,
                                detail::span<TensorViewMut> outputs) {
    std::vector<ClankersTensorView> c_inputs;
    c_inputs.reserve(inputs.size());
    for (const auto& view : inputs) {
        c_inputs.push_back(view.c_view());
    }

    std::vector<ClankersTensorViewMut> c_outputs;
    c_outputs.reserve(outputs.size());
    for (auto& view : outputs) {
        c_outputs.push_back(view.c_view_mut());
    }

    ClankersInferenceStats raw_stats{};
    check(clankers_engine_run_into(
        handle_,
        c_inputs.empty() ? nullptr : c_inputs.data(),
        c_inputs.size(),
        c_outputs.empty() ? nullptr : c_outputs.data(),
        c_outputs.size(),
        &raw_stats));
    return InferenceStats::from_c(raw_stats);
}

}  // namespace clankers
