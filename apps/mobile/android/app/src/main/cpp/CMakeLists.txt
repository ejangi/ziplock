cmake_minimum_required(VERSION 3.22.1)

# Declare the project
project(ziplock_hybrid_jni)

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find required packages
find_library(log-lib log)

# Add the JNI bridge library
add_library(
    ziplock_hybrid_jni
    SHARED
    ziplock_hybrid_jni.cpp
)

# Include directories
target_include_directories(
    ziplock_hybrid_jni
    PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/../../../../../../../shared/include
)

# Link against the prebuilt ziplock_shared library
# add_library(ziplock_shared SHARED IMPORTED)
# set_target_properties(
#     ziplock_shared
#     PROPERTIES
#     IMPORTED_LOCATION ${CMAKE_CURRENT_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}/libziplock_shared.so
# )

# Target link libraries
target_link_libraries(
    ziplock_hybrid_jni
    # ziplock_shared
    ${log-lib}
)

# Compiler flags for optimization and debugging
target_compile_options(
    ziplock_hybrid_jni
    PRIVATE
    -Wall
    -Wextra
    -O2
    -ffunction-sections
    -fdata-sections
)

# Linker flags
target_link_options(
    ziplock_hybrid_jni
    PRIVATE
    -Wl,--gc-sections
    -Wl,--strip-all
)

# Copy the hybrid header file to the build directory
configure_file(
    ${CMAKE_CURRENT_SOURCE_DIR}/../../../../../../../shared/include/ziplock_hybrid.h
    ${CMAKE_CURRENT_BINARY_DIR}/ziplock_hybrid.h
    COPYONLY
)
