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

using TypeData = std::pair<Type, SourceLocation>;

enum struct ScopeKind {
    Invalid,
    Scope,
    IfCondition,
    If,
    Else,
    WhileCondition,
    While,
};

struct IfScopeData {
    size_t ConditionalJumpIp;
    std::vector<TypeData> StackBeforeIf;
};

struct ElseScopeData {
    size_t EndIfJumpIp;
    std::vector<TypeData> StackBeforeElse;
};

struct WhileConditionScopeData {
    size_t JumpToIp;
    std::vector<TypeData> StackBeforeWhile;
};

struct WhileScopeData {
    size_t JumpToIp;
    std::vector<TypeData> StackBeforeWhile;
    size_t ConditionalJumpIp;
};

struct Scope {
    ScopeKind Kind = ScopeKind::Invalid;
    SourceLocation Location;
    std::variant<std::monostate, IfScopeData, ElseScopeData, WhileConditionScopeData, WhileScopeData> Data{};
};

std::vector<Op> CompileOps(std::string_view filepath, std::string_view source) {
    std::vector<Op> ops{};
    std::vector<TypeData> typeStack{};
    std::vector<Scope> scopes{ { .Kind = ScopeKind::Scope } };

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
                auto typeString            = Type_ToString(type);
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
                if (scopes.size() > 1) {
                    auto& scope = scopes.back();
                    fflush(stderr);
                    switch (scope.Kind) {
                        case ScopeKind::Invalid: {
                            assert(false);
                            std::exit(-1);
                        } break;

                        case ScopeKind::Scope: {
                            fprintf(stderr, "Expected a closing } before the end of the file\n");
                        } break;

                        case ScopeKind::IfCondition: {
                            fprintf(stderr, "Expected a { for the if body before the end of the file\n");
                        } break;

                        case ScopeKind::If: {
                            fprintf(stderr, "Expected a closing } for the if body before the end of the file\n");
                        } break;

                        case ScopeKind::Else: {
                            fprintf(stderr, "Expected a closing } for the else body before the end of the file\n");
                        } break;

                        case ScopeKind::WhileCondition: {
                            fprintf(stderr, "Expected a { for the while body before the end of the file\n");
                        } break;

                        case ScopeKind::While: {
                            fprintf(stderr, "Expected a closing } for the while body before the end of the file\n");
                        } break;
                    }
                    fprintf(stderr,
                            "%.*s:%llu:%llu: The scope was opened here\n",
                            STR_FORMAT(scope.Location.Filepath),
                            scope.Location.Line,
                            scope.Location.Column);
                    std::exit(1);
                }

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
                auto& topScope = scopes.back();
                switch (topScope.Kind) {
                    case ScopeKind::IfCondition: {
                        scopes.pop_back();
                        expectTypes({ Type::Bool }, token.Location);
                        typeStack.pop_back();
                        scopes.push_back({
                            .Kind     = ScopeKind::If,
                            .Location = token.Location,
                            .Data =
                                IfScopeData{
                                    .ConditionalJumpIp = ops.size(),
                                    .StackBeforeIf     = typeStack,
                                },
                        });
                        ops.push_back({ .Kind = OpKind::JumpFalse, .Data = 0 });
                    } break;

                    case ScopeKind::WhileCondition: {
                        auto condition = std::move(std::get<WhileConditionScopeData>(topScope.Data));
                        scopes.pop_back();
                        expectTypes({ Type::Bool }, token.Location);
                        typeStack.pop_back();
                        scopes.push_back({
                            .Kind     = ScopeKind::While,
                            .Location = token.Location,
                            .Data =
                                WhileScopeData{
                                    .JumpToIp          = condition.JumpToIp,
                                    .StackBeforeWhile  = std::move(condition.StackBeforeWhile),
                                    .ConditionalJumpIp = ops.size(),
                                },
                        });
                        ops.push_back({ .Kind = OpKind::JumpFalse, .Data = 0 });
                    } break;

                    default: {
                        scopes.push_back({ .Kind = ScopeKind::Scope, .Location = token.Location });
                    } break;
                }
            } break;

            case TokenKind::CloseBrace: {
                bool handled = true;
                if (scopes.size() > 1) {
                    auto scope = std::move(scopes.back());
                    scopes.pop_back();
                    switch (scope.Kind) {
                        case ScopeKind::Invalid: {
                            assert(false);
                            std::exit(-1);
                        } break;

                        case ScopeKind::Scope: {
                        } break;

                        case ScopeKind::If: {
                            auto& data = std::get<IfScopeData>(scope.Data);
                            // TODO: type checking
                            if (lexer.PeekToken().Kind == TokenKind::Else) {
                                token = lexer.NextToken();
                                token = lexer.NextToken();
                                if (token.Kind != TokenKind::OpenBrace) {
                                    fflush(stdout);
                                    auto string = TokenKind_ToString(token.Kind);
                                    fprintf(stderr,
                                            "%.*s:%llu:%llu: Unexpected { for else scope but got %.*s\n",
                                            STR_FORMAT(token.Location.Filepath),
                                            token.Location.Line,
                                            token.Location.Column,
                                            STR_FORMAT(string));
                                    std::exit(1);
                                }
                                scopes.push_back({
                                    .Kind     = ScopeKind::Else,
                                    .Location = token.Location,
                                    .Data =
                                        ElseScopeData{
                                            .EndIfJumpIp     = ops.size(),
                                            .StackBeforeElse = std::move(data.StackBeforeIf),
                                        },
                                });
                                ops.push_back({ .Kind = OpKind::Jump, .Data = 0 });
                                std::get<long long>(ops[data.ConditionalJumpIp].Data) =
                                    static_cast<long long>(ops.size()) - static_cast<long long>(data.ConditionalJumpIp);
                            } else {
                                std::get<long long>(ops[data.ConditionalJumpIp].Data) =
                                    static_cast<long long>(ops.size()) - static_cast<long long>(data.ConditionalJumpIp);
                            }
                        } break;

                        case ScopeKind::Else: {
                            auto& data = std::get<ElseScopeData>(scope.Data);
                            // TODO: type checking
                            std::get<long long>(ops[data.EndIfJumpIp].Data) =
                                static_cast<long long>(ops.size()) - static_cast<long long>(data.EndIfJumpIp);
                        } break;

                        case ScopeKind::While: {
                            auto& data = std::get<WhileScopeData>(scope.Data);
                            // TODO: type checking
                            ops.push_back({
                                .Kind = OpKind::Jump,
                                .Data = static_cast<long long>(data.JumpToIp) - static_cast<long long>(ops.size()),
                            });
                            std::get<long long>(ops[data.ConditionalJumpIp].Data) =
                                static_cast<long long>(ops.size()) - static_cast<long long>(data.ConditionalJumpIp);
                        } break;

                        default: {
                            handled = false;
                        } break;
                    }
                } else {
                    handled = false;
                }

                if (!handled) {
                    fflush(stdout);
                    fprintf(stderr,
                            "%.*s:%llu:%llu: Unexpected }\n",
                            STR_FORMAT(token.Location.Filepath),
                            token.Location.Line,
                            token.Location.Column);
                    std::exit(1);
                }
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
                scopes.push_back({ .Kind = ScopeKind::IfCondition, .Location = token.Location });
            } break;

            case TokenKind::Else: {
                fflush(stdout);
                fprintf(stderr,
                        "%.*s:%llu:%llu: Expected else to be attached to an if\n",
                        STR_FORMAT(token.Location.Filepath),
                        token.Location.Line,
                        token.Location.Column);
                std::exit(1);
            } break;

            case TokenKind::While: {
                scopes.push_back({
                    .Kind     = ScopeKind::WhileCondition,
                    .Location = token.Location,
                    .Data =
                        WhileConditionScopeData{
                            .JumpToIp         = ops.size(),
                            .StackBeforeWhile = typeStack,
                        },
                });
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

    return 0;
}
