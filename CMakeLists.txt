cmake_minimum_required(VERSION 3.20)
project(StackLang)

set(CMAKE_CXX_STANDARD 20)

add_executable(StackLang src/Main.cpp src/Lexer.hpp src/Common.hpp src/Token.hpp src/Lexer.cpp src/Token.cpp src/Ops.hpp src/Ops.cpp src/Types.hpp src/Types.cpp src/Compilation.hpp src/Compilation.cpp src/Execution.hpp src/Execution.cpp)
target_compile_options(StackLang PRIVATE -Wall -Werror)
