#pragma once

#include "Common.hpp"
#include "Token.hpp"

#include <unordered_map>

class Lexer {
private:
    SourceLocation Location;
    std::string_view Source;
private:
    static const std::unordered_map<char, TokenKind> Separators;
    static const std::unordered_map<std::string_view, TokenKind> Keywords;
public:
    Lexer(std::string_view filepath, std::string_view source);
    Token NextToken();
private:
    char CurrentChar();
    char NextChar();
    void SkipWhitespace();
};
