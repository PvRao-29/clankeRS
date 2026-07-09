cmake_minimum_required(VERSION 3.16)
project(clankers_cpp VERSION 0.1.4 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

option(CLANKERS_BUILD_EXAMPLES "Build C++ examples" ON)

if(NOT CLANKERS_FFI_LIB)
    set(CLANKERS_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/..")
    set(CLANKERS_FFI_LIB "${CLANKERS_ROOT}/target/release/libclankers_ffi.a")
endif()

if(NOT EXISTS "${CLANKERS_FFI_LIB}")
    message(FATAL_ERROR
        "clankers FFI library not found at ${CLANKERS_FFI_LIB}. "
        "Run: cargo build -p clankers-ffi --release")
endif()

add_library(clankers_cpp STATIC
    src/error.cpp
    src/stats.cpp
    src/tensor.cpp
    src/engine.cpp
)

target_include_directories(clankers_cpp
    PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:include>
)

target_link_libraries(clankers_cpp PUBLIC "${CLANKERS_FFI_LIB}")

if(APPLE)
    find_library(FOUNDATION_FRAMEWORK Foundation REQUIRED)
    target_link_libraries(clankers_cpp PUBLIC "${FOUNDATION_FRAMEWORK}")
endif()

install(TARGETS clankers_cpp
    EXPORT clankersTargets
    ARCHIVE DESTINATION lib
    LIBRARY DESTINATION lib
)

install(DIRECTORY include/clankers
    DESTINATION include
    FILES_MATCHING PATTERN "*.h" PATTERN "*.hpp"
)

install(EXPORT clankersTargets
    FILE clankersTargets.cmake
    NAMESPACE clankers::
    DESTINATION lib/cmake/clankers
)

include(CMakePackageConfigHelpers)
configure_package_config_file(
    cmake/clankersConfig.cmake.in
    "${CMAKE_CURRENT_BINARY_DIR}/clankersConfig.cmake"
    INSTALL_DESTINATION lib/cmake/clankers
)

install(FILES "${CMAKE_CURRENT_BINARY_DIR}/clankersConfig.cmake"
    DESTINATION lib/cmake/clankers
)

if(CLANKERS_BUILD_EXAMPLES)
    add_executable(minimal_inference examples/minimal_inference.cpp)
    target_link_libraries(minimal_inference PRIVATE clankers_cpp)

    add_executable(zero_alloc_loop examples/zero_alloc_loop.cpp)
    target_link_libraries(zero_alloc_loop PRIVATE clankers_cpp)
endif()
