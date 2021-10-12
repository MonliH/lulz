cmake_minimum_required(VERSION 3.15)

find_program(RUSTC rustc)
find_program(CARGO cargo REQUIRED)

set(CMAKE_BUILD_TYPE Release)
set(CARGO_BUILD_DIR "build")
set(CMAKE_SKIP_INSTALL_ALL_DEPENDENCY TRUE)
project(lulz LANGUAGES C)
add_library(lulzrt src/clib/lol_ops.c src/clib/lol_runtime.c)
set_target_properties(lulzrt PROPERTIES PUBLIC_HEADER "src/clib/lol_ops.h;src/clib/lol_runtime.h")

if (CMAKE_BUILD_TYPE STREQUAL "Release")
    set(RELEASE_FLAG "--release")
    set(BUILD_SUBDIR "release")
else()
    set(RELEASE_FLAG "")
    set(BUILD_SUBDIR "debug")
endif()
set(CARGO_FLAGS ${RELEASE_FLAG})
set(EXEC_OUTPUT_PATH ${CMAKE_CURRENT_SOURCE_DIR}/${CARGO_BUILD_DIR}/${BUILD_SUBDIR}/lulz)
if (CMAKE_BUILD_TYPE STREQUAL "Release")
    set(STRIP_CMD ${CMAKE_STRIP} -s -x ${EXEC_OUTPUT_PATH})
else()
    set(STRIP_CMD "")
endif()

add_custom_target(
    lulz-bin ALL
    COMMAND ${CARGO} build ${CARGO_FLAGS}
    COMMAND ${STRIP_CMD}
    COMMAND ${CMAKE_COMMAND} -E copy ${EXEC_OUTPUT_PATH} ${CMAKE_CURRENT_BINARY_DIR}/lulz
    DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/src/ ${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml
    VERBATIM
)

install(TARGETS lulzrt ARCHIVE PUBLIC_HEADER)
install(
    PROGRAMS ${EXEC_OUTPUT_PATH}
    DESTINATION ${CMAKE_INSTALL_PREFIX}/bin
)

set_property(
  TARGET lulz-bin
  APPEND
  PROPERTY ADDITIONAL_CLEAN_FILES ${CMAKE_CURRENT_SOURCE_DIR}/${CARGO_BUILD_DIR}
)
