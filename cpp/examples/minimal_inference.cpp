#include <clankers/clankers.hpp>

#include <cstdio>
#include <vector>

int main(int argc, char** argv) {
    const char* model_path = argc > 1 ? argv[1]
                                      : "crates/clankers-ml/tests/fixtures/onnx/policy_single_f32.onnx";

    try {
        auto engine = clankers::Engine::builder()
                          .backend("onnxruntime")
                          .model_path(model_path)
                          .build();

        std::printf("clankers %s (ABI %u)\n", clankers_version(), clankers_abi_version());
        std::printf("inputs=%zu outputs=%zu\n", engine.input_count(), engine.output_count());

        std::vector<float> input = {0.1f, 0.2f, 0.3f, 0.4f};
        const clankers::Shape shape({1, 4});
        auto view = clankers::TensorView::from_external(
            clankers::detail::span<const std::byte>(
                reinterpret_cast<const std::byte*>(input.data()),
                input.size() * sizeof(float)),
            clankers::DType::F32,
            shape);

        clankers::InferenceStats stats;
        auto outputs = engine.run_with_stats(clankers::detail::span<const clankers::TensorView>(&view, 1),
                                             stats);
        std::printf("latency_us=%llu copies=%zu allocations=%zu\n",
                    static_cast<unsigned long long>(stats.latency_us),
                    stats.copies,
                    stats.allocations);

        if (!outputs.empty()) {
            const auto bytes = outputs[0].bytes();
            const auto* out_f32 = reinterpret_cast<const float*>(bytes.data());
            const auto n = bytes.size() / sizeof(float);
            std::printf("output[0..%zu):", n);
            for (std::size_t i = 0; i < n; ++i) {
                std::printf(" %.4f", out_f32[i]);
            }
            std::printf("\n");
        }
    } catch (const clankers::Error& err) {
        std::fprintf(stderr, "clankers error: %s\n", err.what());
        return 1;
    }

    return 0;
}
