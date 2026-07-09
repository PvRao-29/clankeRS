#include <clankers/error.hpp>

namespace clankers {

Error::Error(ClankersStatus code, std::string message) : code_(code), message_(std::move(message)) {}

void check(ClankersStatus status) {
    if (status == CLANKERS_STATUS_OK) {
        return;
    }
    const char* msg = clankers_last_error_message();
    if (msg == nullptr) {
        throw Error(status, "clankers call failed");
    }
    throw Error(status, msg);
}

}  // namespace clankers
