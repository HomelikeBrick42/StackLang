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
    If,
    Else,
    While,
    Dup,
    Drop,
};

std::string_view TokenKind_ToString(TokenKind kind);

struct Token {
    TokenKind Kind = TokenKind::Invalid;
    SourceLocation Location{};
    size_t Length{};
    std::variant<std::monostate, long long, std::string_view> Data{};
};
