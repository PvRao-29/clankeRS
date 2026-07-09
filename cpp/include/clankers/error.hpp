#pragma once

#include <clankers/clankers.h>

#include <exception>
#include <string>

namespace clankers {

/// Thrown when a C API call returns a non-OK status.
class Error : public std::exception {
public:
    Error(ClankersStatus code, std::string message);

    ClankersStatus code() const noexcept { return code_; }
    const char* what() const noexcept override { return message_.c_str(); }

private:
    ClankersStatus code_;
    std::string message_;
};

/// Throw `Error` if `status != CLANKERS_STATUS_OK`.
void check(ClankersStatus status);

}  // namespace clankers
