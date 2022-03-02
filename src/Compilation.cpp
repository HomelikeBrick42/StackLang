#include "Compilation.hpp"

#include "Lexer.hpp"
#include "Types.hpp"
#include "Execution.hpp"

#include <span>
#include <cassert>
#include <algorithm>
#include <iterator>
#include <unordered_map>

using TypeData = std::pair<Type, SourceLocation>;

enum struct ScopeKind {
    Invalid,
    Scope,
    IfCondition,
    If,
    Else,
    WhileCondition,
    While,
    Const,
};

struct IfScopeData {
    size_t ConditionalJumpIp{};
    std::vector<Type> StackBeforeIf{};
};

struct ElseScopeData {
    size_t EndIfJumpIp{};
    std::vector<Type> StackBeforeElse{};
};

struct WhileConditionScopeData {
    size_t JumpToIp{};
    std::vector<Type> StackBeforeWhile{};
};

struct WhileScopeData {
    size_t JumpToIp{};
    std::vector<Type> StackBeforeWhile{};
    size_t ConditionalJumpIp{};
};

struct ConstScopeData {
    std::vector<Op> OldOps{};
    std::vector<TypeData> OldTypeStack{};
    std::string_view Name{};
};

struct Scope {
    ScopeKind Kind = ScopeKind::Invalid;
    SourceLocation Location{};
    std::unordered_map<std::string_view, std::vector<std::pair<Type, Op>>> Constants{};
    std::variant<std::monostate, IfScopeData, ElseScopeData, WhileConditionScopeData, WhileScopeData, ConstScopeData> Data{};
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

    auto expectTypes = [&](std::span<const Type> expectedTypes, const SourceLocation& location) {
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

    auto expectTypesLiteral = [&](std::initializer_list<Type> expectedTypes, const SourceLocation& location) {
        expectTypes({ expectedTypes.begin(), expectedTypes.size() }, location);
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

                        case ScopeKind::Const: {
                            fprintf(stderr, "Expected a closing ) for the const value before the end of the file\n");
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

                bool found = false;
                for (auto it = scopes.rbegin(); it != scopes.rend(); it++) {
                    auto& scope = *it;
                    if (scope.Constants.contains(name)) {
                        auto& data = scope.Constants[name];
                        for (auto& [type, op] : data) {
                            ops.push_back(op);
                            typeStack.push_back({type,token.Location});
                        }
                        found = true;
                        break;
                    }
                }

                if (!found) {
                    fflush(stdout);
                    fprintf(stderr,
                            "%.*s:%llu:%llu: Unable to find name %.*s\n",
                            STR_FORMAT(token.Location.Filepath),
                            token.Location.Line,
                            token.Location.Column,
                            STR_FORMAT(name));
                    std::exit(1);
                }
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
                        expectTypesLiteral({ Type::Bool }, token.Location);
                        typeStack.pop_back();
                        std::vector<Type> newStack{};
                        std::transform(typeStack.begin(), typeStack.end(), std::back_inserter(newStack), [](const auto& element) {
                            auto& [type, location] = element;
                            return type;
                        });
                        scopes.push_back({
                            .Kind     = ScopeKind::If,
                            .Location = token.Location,
                            .Data =
                                IfScopeData{
                                    .ConditionalJumpIp = ops.size(),
                                    .StackBeforeIf     = newStack,
                                },
                        });
                        ops.push_back({ .Kind = OpKind::JumpFalse, .Data = 0 });
                    } break;

                    case ScopeKind::WhileCondition: {
                        auto condition = std::move(std::get<WhileConditionScopeData>(topScope.Data));
                        scopes.pop_back();
                        expectTypesLiteral({ Type::Bool }, token.Location);
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
                            if (typeStack.size() != data.StackBeforeIf.size()) {
                                fprintf(stderr,
                                        "%.*s:%llu:%llu: if cannot change the number of elements on the stack\n",
                                        STR_FORMAT(token.Location.Filepath),
                                        token.Location.Line,
                                        token.Location.Column);
                                std::exit(1);
                            }
                            expectTypes(data.StackBeforeIf, token.Location);
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
                            if (typeStack.size() != data.StackBeforeElse.size()) {
                                fprintf(stderr,
                                        "%.*s:%llu:%llu: else cannot change the number of elements on the stack\n",
                                        STR_FORMAT(token.Location.Filepath),
                                        token.Location.Line,
                                        token.Location.Column);
                                std::exit(1);
                            }
                            expectTypes(data.StackBeforeElse, token.Location);
                            std::get<long long>(ops[data.EndIfJumpIp].Data) =
                                static_cast<long long>(ops.size()) - static_cast<long long>(data.EndIfJumpIp);
                        } break;

                        case ScopeKind::While: {
                            auto& data = std::get<WhileScopeData>(scope.Data);
                            if (typeStack.size() != data.StackBeforeWhile.size()) {
                                fprintf(stderr,
                                        "%.*s:%llu:%llu: while cannot change the number of elements on the stack\n",
                                        STR_FORMAT(token.Location.Filepath),
                                        token.Location.Line,
                                        token.Location.Column);
                                std::exit(1);
                            }
                            expectTypes(data.StackBeforeWhile, token.Location);
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

            case TokenKind::OpenParenthesis: {
                fflush(stdout);
                fprintf(stderr,
                        "%.*s:%llu:%llu: Unexpected (\n",
                        STR_FORMAT(token.Location.Filepath),
                        token.Location.Line,
                        token.Location.Column);
                std::exit(1);
            } break;

            case TokenKind::CloseParenthesis: {
                if (std::holds_alternative<ConstScopeData>(scopes.back().Data)) {
                    auto scope = std::move(scopes.back());
                    scopes.pop_back();
                    auto& data = std::get<ConstScopeData>(scope.Data);
                    ops.push_back({ .Kind = OpKind::Exit });
                    auto values = ExecuteOps(ops);
                    std::vector<std::pair<Type, Op>> constantData;
                    for (size_t i = 0; i < values.size(); i++) {
                        Op op;
                        if (std::holds_alternative<long long>(values[i])) {
                            op = {
                                .Kind = OpKind::IntegerPush,
                                .Data = std::get<long long>(values[i]),
                            };
                        } else if (std::holds_alternative<bool>(values[i])) {
                            op = {
                                .Kind = OpKind::BoolPush,
                                .Data = std::get<bool>(values[i]),
                            };
                        }
                        constantData.push_back({ typeStack[i].first, op });
                    }
                    ops       = std::move(data.OldOps);
                    typeStack = std::move(data.OldTypeStack);

                    scopes.back().Constants[data.Name] = constantData;
                } else {
                    fflush(stdout);
                    fprintf(stderr,
                            "%.*s:%llu:%llu: Unexpected )\n",
                            STR_FORMAT(token.Location.Filepath),
                            token.Location.Line,
                            token.Location.Column);
                    std::exit(1);
                }
            } break;

            case TokenKind::Add: {
                expectTypesLiteral({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerAdd });
                typeStack.push_back({ Type::Integer, token.Location });
            } break;

            case TokenKind::Subtract: {
                expectTypesLiteral({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerSubtract });
                typeStack.push_back({ Type::Integer, token.Location });
            } break;

            case TokenKind::Multiply: {
                expectTypesLiteral({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerMultiply });
                typeStack.push_back({ Type::Integer, token.Location });
            } break;

            case TokenKind::Divide: {
                expectTypesLiteral({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerDivide });
                typeStack.push_back({ Type::Integer, token.Location });
            } break;

            case TokenKind::Modulus: {
                expectTypesLiteral({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerModulus });
                typeStack.push_back({ Type::Integer, token.Location });
            } break;

            case TokenKind::Equal: {
                expectTypeCount(2, token.Location);
                auto [type, location] = typeStack.back();
                expectTypesLiteral({ type, type }, token.Location);
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
                expectTypesLiteral({ type, type }, token.Location);
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
                expectTypesLiteral({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerLessThan });
                typeStack.push_back({ Type::Bool, token.Location });
            } break;

            case TokenKind::GreaterThan: {
                expectTypesLiteral({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerGreaterThan });
                typeStack.push_back({ Type::Bool, token.Location });
            } break;

            case TokenKind::LessThanOrEqual: {
                expectTypesLiteral({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerGreaterThan });
                ops.push_back({ .Kind = OpKind::BoolNot });
                typeStack.push_back({ Type::Bool, token.Location });
            } break;

            case TokenKind::GreaterThanOrEqual: {
                expectTypesLiteral({ Type::Integer, Type::Integer }, token.Location);
                typeStack.pop_back();
                typeStack.pop_back();
                ops.push_back({ .Kind = OpKind::IntegerLessThan });
                ops.push_back({ .Kind = OpKind::BoolNot });
                typeStack.push_back({ Type::Bool, token.Location });
            } break;

            case TokenKind::Not: {
                expectTypesLiteral({ Type::Bool }, token.Location);
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
                std::vector<Type> newStack{};
                std::transform(typeStack.begin(), typeStack.end(), std::back_inserter(newStack), [](const auto& element) {
                    auto& [type, location] = element;
                    return type;
                });
                scopes.push_back({
                    .Kind     = ScopeKind::WhileCondition,
                    .Location = token.Location,
                    .Data =
                        WhileConditionScopeData{
                            .JumpToIp         = ops.size(),
                            .StackBeforeWhile = newStack,
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

            case TokenKind::Const: {
                auto constToken = token;
                token           = lexer.NextToken();
                if (token.Kind != TokenKind::Name) {
                    auto string = TokenKind_ToString(token.Kind);
                    fflush(stdout);
                    fprintf(stderr,
                            "%.*s:%llu:%llu: Expected a name for the const, but got %.*s\n",
                            STR_FORMAT(token.Location.Filepath),
                            token.Location.Line,
                            token.Location.Column,
                            STR_FORMAT(string));
                    std::exit(1);
                }
                auto name = std::get<std::string_view>(token.Data);
                token     = lexer.NextToken();
                if (token.Kind != TokenKind::OpenParenthesis) {
                    auto string = TokenKind_ToString(token.Kind);
                    fflush(stdout);
                    fprintf(stderr,
                            "%.*s:%llu:%llu: Expected a ( for the const value, but got %.*s\n",
                            STR_FORMAT(token.Location.Filepath),
                            token.Location.Line,
                            token.Location.Column,
                            STR_FORMAT(string));
                    std::exit(1);
                }
                scopes.push_back({
                    .Kind     = ScopeKind::Const,
                    .Location = constToken.Location,
                    .Data =
                        ConstScopeData{
                            .OldOps       = std::move(ops),
                            .OldTypeStack = std::move(typeStack),
                            .Name         = name,
                        },
                });
                ops       = {};
                typeStack = {};
            } break;
        }
    }
EndLoop:

    return ops;
}
