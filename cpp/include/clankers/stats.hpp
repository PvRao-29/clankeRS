#pragma once

#include <clankers/clankers.h>

#include <cstddef>
#include <cstdint>

namespace clankers {

/// Per-run inference accounting from the C ABI.
struct InferenceStats {
    std::uint64_t latency_us = 0;
    std::size_t copies = 0;
    std::size_t bytes_copied = 0;
    std::size_t allocations = 0;
    std::size_t bytes_allocated = 0;
    std::uint64_t backend_latency_us = 0;
    std::size_t backend_copies = 0;
    std::size_t backend_bytes_copied = 0;

    static InferenceStats from_c(const ClankersInferenceStats& raw);
    bool is_zero_copy() const noexcept { return copies == 0; }
};

}  // namespace clankers
