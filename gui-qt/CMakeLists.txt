cmake_minimum_required(VERSION 3.22)
project(hashcalc-qt LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Qt6 static installation
list(PREPEND CMAKE_PREFIX_PATH "/home/antidote/qt6-static")

find_package(Qt6 6.6 REQUIRED COMPONENTS Widgets)
qt_standard_project_setup()

# ── Rust FFI library ──────────────────────────────────────────────────────────

set(WORKSPACE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/..")
set(RUST_LIB "${WORKSPACE_DIR}/target/release/libhashcalc_ffi.a")
set(RUST_FFI_INCLUDE "${WORKSPACE_DIR}/ffi/include")

find_program(CARGO cargo REQUIRED)

add_custom_command(
    OUTPUT  "${RUST_LIB}"
    COMMAND "${CARGO}" build --release -p hashcalc-ffi
    WORKING_DIRECTORY "${WORKSPACE_DIR}"
    COMMENT "Building hashcalc-ffi (Rust staticlib)"
    VERBATIM
)
add_custom_target(build_hashcalc_ffi ALL DEPENDS "${RUST_LIB}")

add_library(hashcalc_ffi_lib STATIC IMPORTED GLOBAL)
set_target_properties(hashcalc_ffi_lib PROPERTIES IMPORTED_LOCATION "${RUST_LIB}")
add_dependencies(hashcalc_ffi_lib build_hashcalc_ffi)

# ── Qt executable ─────────────────────────────────────────────────────────────

qt_add_executable(hashcalc-qt
    src/main.cpp
    src/MainWindow.cpp
    src/DropZone.cpp
)

add_dependencies(hashcalc-qt build_hashcalc_ffi)

target_include_directories(hashcalc-qt PRIVATE "${RUST_FFI_INCLUDE}")

target_link_libraries(hashcalc-qt PRIVATE
    Qt6::Widgets
    hashcalc_ffi_lib
    pthread
    dl
    m
)

# Static Qt: import platform plugins
qt_import_plugins(hashcalc-qt
    INCLUDE Qt6::QXcbIntegrationPlugin
)
