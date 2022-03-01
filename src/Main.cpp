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
#include <vector>

#define STR_FORMAT(str) static_cast<int>((str).size()), (str).data()

enum struct OpKind {
    Invalid,
    Exit,
    Jump,
    JumpFalse,
    IntegerPush,
    IntegerDup,
    IntegerDrop,
    IntegerAdd,
    IntegerSubtract,
    IntegerMultiply,
    IntegerDivide,
    IntegerModulus,
    IntegerLessThan,
    IntegerGreaterThan,
    IntegerEqual,
    IntegerPrint,
    BoolPush,
    BoolDup,
    BoolDrop,
    BoolNot,
    BoolEqual,
    BoolPrint,
};

std::string_view OpKind_ToString(OpKind kind) {
    switch (kind) {
        case OpKind::Invalid:
            return "Invalid";
        case OpKind::Exit:
            return "Exit";
        case OpKind::Jump:
            return "Jump";
        case OpKind::JumpFalse:
            return "JumpFalse";
        case OpKind::IntegerPush:
            return "IntegerPush";
        case OpKind::IntegerDup:
            return "IntegerDup";
        case OpKind::IntegerDrop:
            return "IntegerDrop";
        case OpKind::IntegerAdd:
            return "IntegerAdd";
        case OpKind::IntegerSubtract:
            return "IntegerSubtract";
        case OpKind::IntegerMultiply:
            return "IntegerMultiply";
        case OpKind::IntegerDivide:
            return "IntegerDivide";
        case OpKind::IntegerModulus:
            return "IntegerModulus";
        case OpKind::IntegerLessThan:
            return "IntegerLessThan";
        case OpKind::IntegerGreaterThan:
            return "IntegerGreaterThan";
        case OpKind::IntegerEqual:
            return "IntegerEqual";
        case OpKind::IntegerPrint:
            return "IntegerPrint";
        case OpKind::BoolPush:
            return "BoolPush";
        case OpKind::BoolDup:
            return "BoolDup";
        case OpKind::BoolDrop:
            return "BoolDrop";
        case OpKind::BoolNot:
            return "BoolNot";
        case OpKind::BoolEqual:
            return "BoolEqual";
        case OpKind::BoolPrint:
            return "BoolPrint";
    }
    assert(false);
    std::exit(-1);
}

struct Op {
    OpKind Kind = OpKind::Invalid;
    std::variant<std::monostate, long long, bool> Data{};
};

enum struct Type {
    Invalid,
    Integer,
    Bool,
};

std::string_view Type_ToString(Type type) {
    switch (type) {
        case Type::Invalid:
            return "Invalid";
        case Type::Integer:
            return "Integer";
        case Type::Bool:
            return "Bool";
    }
    assert(false);
    std::exit(-1);
}

std::vector<Op> CompileOps(std::string_view filepath, std::string_view source) {
    std::vector<Op> ops{};
    std::vector<std::pair<Type, SourceLocation>> typeStack{};

    auto expectTypeCount = [&](size_t count, const SourceLocation& location) {
        if (typeStack.size() < count) {
            fflush(stdout);
            fprintf(stderr,
                    "%.*s:%llu:%llu: Expected at least %llu elements on the stack\n",
                    STR_FORMAT(location.Filepath),
                    location.Line,
                    location.Column,
                    count);
            std::exit(-1);
        }
    };

    auto expectTypes = [&](std::initializer_list<Type> expectedTypes, const SourceLocation& location) {
        expectTypeCount(expectedTypes.size(), location);
        bool equal = true;
        for (size_t i = 0; i < expectedTypes.size(); i++) {
            auto& expectedType                     = *(std::rbegin(expectedTypes) + i);
            auto& [actualType, actualTypeLocation] = *(std::rbegin(typeStack) + i);
            if (expectedType != actualType) {
                equal = false;
                break;
            }
        }
        if (!equal) {
            fflush(stdout);
            fprintf(stderr,
                    "%.*s:%llu:%llu: Incorrect types on the stack\n",
                    STR_FORMAT(location.Filepath),
                    location.Line,
                    location.Column);
            fprintf(stderr, "Expected:\n");
            for (size_t i = 0; i < expectedTypes.size(); i++) {
                auto typeString = Type_ToString(*(std::rbegin(expectedTypes) + i));
                fprintf(stderr, "    %.*s\n", STR_FORMAT(typeString));
            }
            fprintf(stderr, "But got:\n");
            for (size_t i = 0; i < expectedTypes.size(); i++) {
                auto& [type, typeLocation] = *(std::rbegin(typeStack) + i);
                auto typeString = Type_ToString(type);
                fprintf(stderr,
                        "    %.*s:%llu:%llu: %.*s\n",
                        STR_FORMAT(typeLocation.Filepath),
                        typeLocation.Line,
                        typeLocation.Column,
                        STR_FORMAT(typeString));
            }
            std::exit(1);
        }
    };

    Lexer lexer{ filepath, source };
    while (true) {
        auto token = lexer.NextToken();
        switch (token.Kind) {
            case TokenKind::Invalid: {
                assert(false);
                std::exit(-1);
            } break;

            case TokenKind::EndOfFile: {
                ops.push_back({ .Kind = OpKind::Exit });
                if (typeStack.size() > 0) {
                    fflush(stdout);
                    fprintf(stderr, "There must be no elements on the stack at the end of the program\n");
                    fprintf(stderr, "The elements are:\n");
                    while (typeStack.size() > 0) {
                        auto [type, location] = typeStack.back();
                        typeStack.pop_back();
                        auto typeString = Type_ToString(type);
                        fprintf(stderr,
                                "    %.*s:%llu:%llu: %.*s\n",
                                STR_FORMAT(location.Filepath),
                                location.Line,
                                location.Column,
                                STR_FORMAT(typeString));
                    }
                    std::exit(1);
                }
                goto EndLoop;
            } break;

            case TokenKind::Name: {
                auto name = std::get<std::string_view>(token.Data);

                fflush(stdout);
                fprintf(stderr,
                        "%.*s:%llu:%llu: Unable to find name %.*s\n",
                        STR_FORMAT(token.Location.Filepath),
                        token.Location.Line,
                        token.Location.Column,
                        STR_FORMAT(name));
                std::exit(1);
            } break;

            case TokenKind::Integer: {
                auto value = std::get<long long>(token.Data);
                ops.push_back({ .Kind = OpKind::IntegerPush, .Data = value });
                typeStack.push_back({ Type::Integer, token.Location });
            } break;

            case TokenKind::OpenBrace: {
            } break;

            case TokenKind::CloseBrace: {
            } break;

            case TokenKind::Add: {
                expectTypes({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerAdd });
                typeStack.push_back({ Type::Integer, token.Location });
            } break;

            case TokenKind::Subtract: {
                expectTypes({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerSubtract });
                typeStack.push_back({ Type::Integer, token.Location });
            } break;

            case TokenKind::Multiply: {
                expectTypes({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerMultiply });
                typeStack.push_back({ Type::Integer, token.Location });
            } break;

            case TokenKind::Divide: {
                expectTypes({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerDivide });
                typeStack.push_back({ Type::Integer, token.Location });
            } break;

            case TokenKind::Modulus: {
                expectTypes({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerModulus });
                typeStack.push_back({ Type::Integer, token.Location });
            } break;

            case TokenKind::Equal: {
                expectTypeCount(2, token.Location);
                auto [type, location] = typeStack.back();
                expectTypes({ type, type }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                switch (type) {
                    case Type::Integer: {
                        ops.push_back({ .Kind = OpKind::IntegerEqual });
                    } break;

                    case Type::Bool: {
                        ops.push_back({ .Kind = OpKind::BoolEqual });
                    } break;

                    default: {
                        auto typeString = Type_ToString(type);
                        fflush(stdout);
                        fprintf(stderr,
                                "%.*s:%llu:%llu: Unable to check equality for type %.*s\n",
                                STR_FORMAT(location.Filepath),
                                location.Line,
                                location.Column,
                                STR_FORMAT(typeString));
                        std::exit(1);
                    } break;
                }
                typeStack.push_back({ Type::Bool, token.Location });
            } break;

            case TokenKind::NotEqual: {
                expectTypeCount(2, token.Location);
                auto [type, location] = typeStack.back();
                expectTypes({ type, type }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                switch (type) {
                    case Type::Integer: {
                        ops.push_back({ .Kind = OpKind::IntegerEqual });
                        ops.push_back({ .Kind = OpKind::BoolNot });
                    } break;

                    case Type::Bool: {
                        ops.push_back({ .Kind = OpKind::BoolEqual });
                        ops.push_back({ .Kind = OpKind::BoolNot });
                    } break;

                    default: {
                        auto typeString = Type_ToString(type);
                        fflush(stdout);
                        fprintf(stderr,
                                "%.*s:%llu:%llu: Unable to check non-equality for type %.*s\n",
                                STR_FORMAT(location.Filepath),
                                location.Line,
                                location.Column,
                                STR_FORMAT(typeString));
                        std::exit(1);
                    } break;
                }
                typeStack.push_back({ Type::Bool, token.Location });
            } break;

            case TokenKind::LessThan: {
                expectTypes({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerLessThan });
                typeStack.push_back({ Type::Bool, token.Location });
            } break;

            case TokenKind::GreaterThan: {
                expectTypes({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerGreaterThan });
                typeStack.push_back({ Type::Bool, token.Location });
            } break;

            case TokenKind::LessThanOrEqual: {
                expectTypes({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerGreaterThan });
                ops.push_back({ .Kind = OpKind::BoolNot });
                typeStack.push_back({ Type::Bool, token.Location });
            } break;

            case TokenKind::GreaterThanOrEqual: {
                expectTypes({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerLessThan });
                ops.push_back({ .Kind = OpKind::BoolNot });
                typeStack.push_back({ Type::Bool, token.Location });
            } break;

            case TokenKind::Not: {
                expectTypes({ Type::Bool }, token.Location);
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::BoolNot });
                typeStack.push_back({ Type::Bool, token.Location });
            } break;

            case TokenKind::Print: {
                expectTypeCount(1, token.Location);
                auto [type, location] = typeStack.back();
                typeStack.pop_back();
                switch (type) {
                    case Type::Integer: {
                        ops.push_back({ .Kind = OpKind::IntegerPrint });
                    } break;

                    case Type::Bool: {
                        ops.push_back({ .Kind = OpKind::BoolPrint });
                    } break;

                    default: {
                        auto typeString = Type_ToString(type);
                        fflush(stdout);
                        fprintf(stderr,
                                "%.*s:%llu:%llu: Unable to print type %.*s\n",
                                STR_FORMAT(location.Filepath),
                                location.Line,
                                location.Column,
                                STR_FORMAT(typeString));
                        std::exit(1);
                    } break;
                }
            } break;

            case TokenKind::If: {
            } break;

            case TokenKind::Else: {
            } break;

            case TokenKind::While: {
            } break;

            case TokenKind::Dup: {
                expectTypeCount(1, token.Location);
                auto [type, location] = typeStack.back();
                switch (type) {
                    case Type::Integer: {
                        ops.push_back({ .Kind = OpKind::IntegerDup });
                    } break;

                    case Type::Bool: {
                        ops.push_back({ .Kind = OpKind::BoolDup });
                    } break;

                    default: {
                        auto typeString = Type_ToString(type);
                        fflush(stdout);
                        fprintf(stderr,
                                "%.*s:%llu:%llu: Unable to duplicate type %.*s\n",
                                STR_FORMAT(location.Filepath),
                                location.Line,
                                location.Column,
                                STR_FORMAT(typeString));
                        std::exit(1);
                    } break;
                }
                typeStack.push_back({ type, token.Location });
            } break;

            case TokenKind::Drop: {
                expectTypeCount(1, token.Location);
                auto [type, location] = typeStack.back();
                typeStack.pop_back();
                switch (type) {
                    case Type::Integer: {
                        ops.push_back({ .Kind = OpKind::IntegerDrop });
                    } break;

                    case Type::Bool: {
                        ops.push_back({ .Kind = OpKind::BoolDrop });
                    } break;

                    default: {
                        auto typeString = Type_ToString(type);
                        fflush(stdout);
                        fprintf(stderr,
                                "%.*s:%llu:%llu: Unable to drop type %.*s\n",
                                STR_FORMAT(location.Filepath),
                                location.Line,
                                location.Column,
                                STR_FORMAT(typeString));
                        std::exit(1);
                    } break;
                }
            } break;
        }
    }
EndLoop:

    return ops;
}

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
    for (auto& op : ops) {
        auto string = OpKind_ToString(op.Kind);
        printf("%.*s\n", STR_FORMAT(string));
    }

    return 0;
}
