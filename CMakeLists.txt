cmake_minimum_required(VERSION 3.0)
project(gerlib VERSION 0.1 LANGUAGES CXX C)

#########################################################################################
# Configs
#########################################################################################
set(CMAKE_VERBOSE_MAKEFILE OFF)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_CXX_FLAGS "-fsanitize=undefined -Wall")
set(CMAKE_CXX_FLAGS_DEBUG "-g -ggdb")
set(CMAKE_CXX_FLAGS_RELEASE "-O2 -DNDEBUG")

#########################################################################################
# Global Defines
#########################################################################################
set(GERLIB_LIBRARY "gerlib")
set(GERCAPNP_LIBRARY "ger_capnp")

#########################################################################################
# Dependencies
#########################################################################################
# Conan
include(${CMAKE_BINARY_DIR}/conanbuildinfo.cmake)
conan_basic_setup()
include_directories(${CONAN_INCLUDE_DIRS})
add_definitions(${CONAN_DEFINES})
link_libraries(${CONAN_LIBS})
# FMT
include_directories(third-party/fmt/include)
add_subdirectory(third-party/fmt)
link_libraries(fmt)

#########################################################################################
# Cap'n Proto
#########################################################################################
add_subdirectory(capnp)

#########################################################################################
# Library
#########################################################################################