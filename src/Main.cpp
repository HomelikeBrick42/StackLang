#include "Common.hpp"
#include "Lexer.hpp"
#include "Ops.hpp"
#include "Compilation.hpp"
#include "Execution.hpp"

#include <fstream>
#include <sstream>
#include <string>
#include <cassert>

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

#if 0
    Lexer lexer{ filepath, source };
    while (true) {
        auto token = lexer.NextToken();

        auto string = TokenKind_ToString(token.Kind);
        printf("%.*s\n", STR_FORMAT(string));

        if (token.Kind == TokenKind::EndOfFile)
            break;
    }
#endif

    auto ops = CompileOps(filepath, source);

#if 0
    for (size_t i = 0; i < ops.size(); i++) {
        auto& op    = ops[i];
        auto string = OpKind_ToString(op.Kind);
        fprintf(stdout, "%llu: %.*s", i, STR_FORMAT(string));
        if (std::holds_alternative<long long>(op.Data)) {
            fprintf(stdout, " %lld", std::get<long long>(op.Data));
        } else if (std::holds_alternative<bool>(op.Data)) {
            fprintf(stdout, " %s", std::get<bool>(op.Data) ? "true" : "false");
        }
        fputc('\n', stdout);
    }
    fputc('\n', stdout);
    fputc('\n', stdout);
#endif

    ExecuteOps(ops);

    return 0;
}
