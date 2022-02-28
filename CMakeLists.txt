cmake_minimum_required(VERSION 3.20)
project(StackLang)

set(CMAKE_CXX_STANDARD 20)

add_executable(StackLang src/Main.cpp src/Lexer.hpp src/Common.hpp src/Token.hpp src/Lexer.cpp src/Token.cpp)
target_compile_options(StackLang PRIVATE -Wall -Werror)
