#pragma once

#include "Common.hpp"

#include <variant>

enum struct TokenKind {
    Invalid,
    EndOfFile,
    Name,
    Integer,
    OpenBrace,
    CloseBrace,
    OpenParenthesis,
    CloseParenthesis,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    Not,
    Print,
    Pointer,
    Dereference,
    AssignLeft,
    AssignRight,
    If,
    Else,
    While,
    Dup,
    Drop,
    Const,
};

std::string_view TokenKind_ToString(TokenKind kind);

struct Token {
    TokenKind Kind = TokenKind::Invalid;
    SourceLocation Location{};
    size_t Length{};
    std::variant<std::monostate, long long, std::string_view> Data{};
};
