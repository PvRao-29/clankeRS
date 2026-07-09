#include <clankers/stats.hpp>

namespace clankers {

InferenceStats InferenceStats::from_c(const ClankersInferenceStats& raw) {
    InferenceStats stats;
    stats.latency_us = raw.latency_us;
    stats.copies = raw.copies;
    stats.bytes_copied = raw.bytes_copied;
    stats.allocations = raw.allocations;
    stats.bytes_allocated = raw.bytes_allocated;
    stats.backend_latency_us = raw.backend_latency_us;
    stats.backend_copies = raw.backend_copies;
    stats.backend_bytes_copied = raw.backend_bytes_copied;
    return stats;
}

}  // namespace clankers
