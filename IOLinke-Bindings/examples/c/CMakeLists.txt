cmake_minimum_required(VERSION 3.10)
project(iolinke_c_example C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Set the path to the static library
set(IOLINKE_LIB_PATH "${CMAKE_SOURCE_DIR}/../../../target/debug/libiolinke_bindings.a")

# Set the include directory paths
set(IOLINKE_INCLUDE_PATH "${CMAKE_SOURCE_DIR}/../../../target/include/")

# Add executable
add_executable(iolinke_example main.c)

# Add include directory
target_include_directories(iolinke_example PRIVATE ${IOLINKE_INCLUDE_PATH})

# Link against the static library
target_link_libraries(iolinke_example PRIVATE ${IOLINKE_LIB_PATH})
