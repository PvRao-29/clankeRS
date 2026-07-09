#include <clankers/clankers.hpp>

#include <cstdio>
#include <vector>

/// Demonstrates a zero-allocation inference hot loop via `run_into`.
int main() {
    try {
        const std::size_t dims[] = {1, 4};
        auto engine = clankers::Engine::builder()
                          .backend("noop")
                          .noop_shape(clankers::detail::span<const std::size_t>(dims, 2))
                          .strict_realtime(true)
                          .build();

        std::vector<float> input = {1.f, 2.f, 3.f, 4.f};
        std::vector<float> output(4, 0.f);
        const clankers::Shape shape({1, 4});

        auto in_view = clankers::TensorView::from_external(
            clankers::detail::span<const std::byte>(
                reinterpret_cast<const std::byte*>(input.data()),
                input.size() * sizeof(float)),
            clankers::DType::F32,
            shape);

        auto out_view = clankers::TensorViewMut::from_external(
            clankers::detail::span<std::byte>(
                reinterpret_cast<std::byte*>(output.data()),
                output.size() * sizeof(float)),
            clankers::DType::F32,
            shape);

        clankers::TensorView in_span_storage[] = {in_view};
        clankers::TensorViewMut out_span_storage[] = {out_view};

        for (int i = 0; i < 1000; ++i) {
            auto stats = engine.run_into(
                clankers::detail::span<const clankers::TensorView>(in_span_storage, 1),
                clankers::detail::span<clankers::TensorViewMut>(out_span_storage, 1));
            if (stats.allocations != 0 || stats.copies != 0) {
                std::fprintf(stderr,
                             "iteration %d: allocations=%zu copies=%zu\n",
                             i,
                             stats.allocations,
                             stats.copies);
                return 1;
            }
        }

        std::printf("1000 run_into iterations: 0 allocations, 0 copies\n");
        std::printf("output matches input: %s\n", output == input ? "yes" : "no");
        return output == input ? 0 : 1;
    } catch (const clankers::Error& err) {
        std::fprintf(stderr, "clankers error: %s\n", err.what());
        return 1;
    }
}
