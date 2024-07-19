cmake_minimum_required(VERSION 3.18)
project(wasmi C)

set(WASMI_USER_CARGO_BUILD_OPTIONS "" CACHE STRING "Additional cargo flags (such as --features) to apply to the build command")
option(BUILD_SHARED_LIBS "Build using shared libraries" OFF)
option(WASMI_ALWAYS_BUILD "If cmake should always invoke cargo to build Wasmi" ON)
set(WASMI_TARGET "" CACHE STRING "Rust target to build for")

if(NOT WASMI_TARGET)
    execute_process(
        COMMAND rustc -vV
        OUTPUT_VARIABLE RUSTC_VERSION
    )
    string(REGEX MATCH "host: ([^ \n]+)" RUSTC_VERSION_MATCH "${RUSTC_VERSION}")
    string(STRIP "${CMAKE_MATCH_1}" WASMI_TARGET)
endif()

# If the Wasmi Rust crate shall be built using debug or release settings.
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(WASMI_BUILD_TYPE "debug")
    set(CARGO_PROFILE_PANIC CARGO_PROFILE_DEBUG_PANIC)
else()
    set(WASMI_BUILD_TYPE_FLAG "--profile" "bench")
    set(WASMI_BUILD_TYPE "release")
    set(CARGO_PROFILE_PANIC CARGO_PROFILE_RELEASE_PANIC)
endif()

# Sets the Wasmi target directory.
set(WASMI_TARGET_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../../target/${WASMI_TARGET}/${WASMI_BUILD_TYPE})
if(WASMI_TARGET MATCHES "darwin")
    set(WASMI_SHARED_FILES libwasmi.dylib)
    set(WASMI_STATIC_FILES libwasmi.a)
elseif(WASMI_TARGET MATCHES "windows-gnu")
    set(WASMI_SHARED_FILES libwasmi.dll.a wasmi.dll)
    set(WASMI_STATIC_FILES libwasmi.a)
elseif(WASMI_TARGET MATCHES "windows-msvc")
    set(WASMI_SHARED_FILES wasmi.dll.lib wasmi.dll)
    set(WASMI_STATIC_FILES wasmi.lib)
else()
    set(WASMI_SHARED_FILES libwasmi.so)
    set(WASMI_STATIC_FILES libwasmi.a)
endif()
list(TRANSFORM WASMI_SHARED_FILES PREPEND ${WASMI_TARGET_DIR}/)
list(TRANSFORM WASMI_STATIC_FILES PREPEND ${WASMI_TARGET_DIR}/)

# Instructions on how to build and install the Wasmi Rust crate.
find_program(WASMI_CARGO_BINARY cargo REQUIRED)
include(ExternalProject)
ExternalProject_Add(
    wasmi-crate
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    INSTALL_COMMAND "${WASMI_INSTALL_COMMAND}"
    BUILD_COMMAND
        ${CMAKE_COMMAND} -E env ${CARGO_PROFILE_PANIC}=abort
        ${WASMI_CARGO_BINARY} build
          --package wasmi_c_api
          --target ${WASMI_TARGET}
          --no-default-features
          ${WASMI_BUILD_TYPE_FLAG}
          ${WASMI_FEATURES}
          ${WASMI_USER_CARGO_BUILD_OPTIONS}
    USES_TERMINAL_BUILD TRUE
    BINARY_DIR ${CMAKE_CURRENT_SOURCE_DIR}/artifact
    BUILD_ALWAYS ${WASMI_ALWAYS_BUILD}
    BUILD_BYPRODUCTS ${WASMI_SHARED_FILES} ${WASMI_STATIC_FILES}
)
add_library(wasmi INTERFACE)
add_dependencies(wasmi wasmi-crate)

# Handle platform-specific settings for linking
if(BUILD_SHARED_LIBS)
    if(NOT WASMI_TARGET MATCHES "windows")
        set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -Wl,-rpath='$ORIGIN'")
    endif()
    list(GET WASMI_SHARED_FILES 0 WASMI_SHARED_LIB_TO_LINK)
    target_link_libraries(wasmi INTERFACE ${WASMI_SHARED_LIB_TO_LINK})
else()
    target_link_libraries(wasmi INTERFACE ${WASMI_STATIC_FILES})

    if(WASMI_TARGET MATCHES "windows")
        target_compile_options(wasmi INTERFACE -DWASM_API_EXTERN= -DWASI_API_EXTERN=)
        target_link_libraries(wasmi INTERFACE ws2_32 advapi32 userenv ntdll shell32 ole32 bcrypt)
    elseif(NOT WASMI_TARGET MATCHES "darwin")
        target_link_libraries(wasmi INTERFACE pthread dl m)
    endif()
endif()

target_include_directories(wasmi INTERFACE
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${CMAKE_BINARY_DIR}/include
)

set(WASMI_GENERATED_CONF_H ${CMAKE_BINARY_DIR}/include/wasmi/conf.h)
configure_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/include/wasmi/conf.h.in
    ${WASMI_GENERATED_CONF_H}
)

include(GNUInstallDirs)
install(
    DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}/include/
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
    FILES_MATCHING PATTERN "*.h" PATTERN "*.hh"
)
install(
    FILES ${WASMI_GENERATED_CONF_H}
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}/wasmi
)
install(
    FILES ${WASMI_SHARED_FILES} ${WASMI_STATIC_FILES}
    DESTINATION ${CMAKE_INSTALL_LIBDIR}
)

if(WASMI_TARGET MATCHES "darwin")
    set(INSTALLED_LIB "${CMAKE_INSTALL_PREFIX}/${CMAKE_INSTALL_LIBDIR}/libwasmi.dylib")
    install(
        FILES "${INSTALLED_LIB}"
        DESTINATION "${CMAKE_INSTALL_PREFIX}/${CMAKE_INSTALL_LIBDIR}"
        COMPONENT Runtime
    )
    if(NOT CMAKE_INSTALL_NAME_TOOL)
        message(WARNING "CMAKE_INSTALL_NAME_TOOL is not set. LC_ID_DYLIB for libwasmi.dylib will not be set.")
    else()
        set(install_name_tool_cmd
            "${CMAKE_INSTALL_NAME_TOOL}"
            "-id"
            "@rpath/libwasmi.dylib"
            "${INSTALLED_LIB}"
        )
        install(CODE "execute_process(COMMAND ${install_name_tool_cmd})")
    endif()
endif()

# Documentation Generation via Doxygen:
set(DOXYGEN_CONF_IN ${CMAKE_CURRENT_SOURCE_DIR}/doxygen.conf.in)
set(DOXYGEN_CONF_OUT ${CMAKE_BINARY_DIR}/doxygen.conf)
configure_file(${DOXYGEN_CONF_IN} ${DOXYGEN_CONF_OUT})
add_custom_target(doc
    COMMAND doxygen ${DOXYGEN_CONF_OUT}
    DEPENDS ${WASMI_GENERATED_CONF_H} ${DOXYGEN_CONF_OUT}
    WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
)

# C-Header Formatting via clang-format:
find_program(CLANG_FORMAT clang-format REQUIRED)
file(GLOB_RECURSE HEADER_FILES
    ${CMAKE_CURRENT_SOURCE_DIR}/include/wasmi.h
    ${CMAKE_CURRENT_SOURCE_DIR}/include/wasmi/*.h
    ${CMAKE_CURRENT_SOURCE_DIR}/include/wasmi/*.hh
)
add_custom_target(check-format
    COMMAND ${CLANG_FORMAT} -style=llvm -Werror --dry-run ${HEADER_FILES}
    COMMENT "clang-format: Check formatting for Wasmi C-API header files"
)
add_custom_target(format
    COMMAND ${CLANG_FORMAT} -style=llvm -i ${HEADER_FILES}
    COMMENT "clang-format: Apply formatting rules for Wasmi C-API header files"
)
