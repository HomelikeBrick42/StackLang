#include "Common.hpp"
#include "Lexer.hpp"

#include <iostream>
#include <string_view>
#include <fstream>
#include <sstream>
#include <string>
#include <cstdlib>
#include <variant>
#include <cassert>
#include <cstdint>

#define STR_FORMAT(str) static_cast<int>((str).size()), (str).data()

int main(int, char** argv) {
    assert(*argv);
    std::string_view programName = *argv++;

    if (!*argv) {
        fflush(stdout);
        fprintf(stderr, "Usage: %.*s <file>\n", STR_FORMAT(programName));
        std::exit(1);
    }

    std::string_view filepath = *argv++;

    if (*argv) {
        fflush(stdout);
        fprintf(stderr, "Usage: %.*s <file>\n", STR_FORMAT(programName));
        std::exit(1);
    }

    auto source = [&]() -> std::string {
        std::fstream file{ std::string{ filepath } };
        if (!file.is_open()) {
            fflush(stdout);
            fprintf(stderr, "Unable to open file '%.*s'\n", STR_FORMAT(filepath));
            std::exit(1);
        }
        std::stringstream ss{};
        ss << file.rdbuf();
        return ss.str();
    }();

    Lexer lexer{ filepath, source };
    while (true) {
        auto token = lexer.NextToken();

        auto string = TokenKind_ToString(token.Kind);
        printf("%.*s\n", STR_FORMAT(string));

        if (token.Kind == TokenKind::EndOfFile)
            break;
    }

    return 0;
}
