#include "Lexer.hpp"

#include <string>
#include <stdexcept>

const std::unordered_map<char, TokenKind> Lexer::Separators{
    { '{', TokenKind::OpenBrace },        { '}', TokenKind::CloseBrace }, { '(', TokenKind::OpenParenthesis },
    { ')', TokenKind::CloseParenthesis }, { '@', TokenKind::Pointer },    { '^', TokenKind::Dereference },
};

const std::unordered_map<std::string_view, TokenKind> Lexer::Keywords{
    { "+", TokenKind::Add },
    { "-", TokenKind::Subtract },
    { "*", TokenKind::Multiply },
    { "/", TokenKind::Divide },
    { "%", TokenKind::Modulus },
    { "==", TokenKind::Equal },
    { "!=", TokenKind::NotEqual },
    { "<", TokenKind::LessThan },
    { ">", TokenKind::GreaterThan },
    { "<=", TokenKind::LessThanOrEqual },
    { ">=", TokenKind::GreaterThanOrEqual },
    { "!", TokenKind::Not },
    { "print", TokenKind::Print },
    { "<-", TokenKind::AssignLeft },
    { "->", TokenKind::AssignRight },
    { "if", TokenKind::If },
    { "else", TokenKind::Else },
    { "while", TokenKind::While },
    { "dup", TokenKind::Dup },
    { "drop", TokenKind::Drop },
    { "const", TokenKind::Const },
};

Lexer::Lexer(std::string_view filepath, std::string_view source)
    : Location{ .Filepath = filepath, .Position = 0, .Line = 1, .Column = 1 }, Source{ source } {}

Token Lexer::NextToken() {
Start:
    SkipWhitespace();
    SourceLocation startLocation = Location;
    if (CurrentChar() == '\0') {
        return {
            .Kind     = TokenKind::EndOfFile,
            .Location = startLocation,
            .Length   = Location.Position - startLocation.Position,
        };
    }
    while (!std::isspace(CurrentChar()) && !Separators.contains(CurrentChar()))
        NextChar();
    if ((Location.Position - startLocation.Position) == 0) {
        auto kind = Separators.at(NextChar());
        return {
            .Kind     = kind,
            .Location = startLocation,
            .Length   = Location.Position - startLocation.Position,
        };
    } else {
        size_t length = Location.Position - startLocation.Position;
        auto name     = Source.substr(startLocation.Position, length);

        try {
            size_t index;
            auto value = std::stoll(std::string{ name }, &index);
            if (index != name.length())
                throw std::invalid_argument{ "" };
            return {
                .Kind     = TokenKind::Integer,
                .Location = startLocation,
                .Length   = length,
                .Data     = value,
            };
        } catch (std::invalid_argument) {
            if (Keywords.contains(name)) {
                auto kind = Keywords.at(name);
                return {
                    .Kind     = kind,
                    .Location = startLocation,
                    .Length   = length,
                };
            } else if (name == "/*") {
                while (true) {
                    Token token = NextToken();
                    if (token.Kind == TokenKind::EndOfFile || Source.substr(token.Location.Position, token.Length) == "*/") {
                        goto Start;
                    }
                }
            } else {
                return {
                    .Kind     = TokenKind::Name,
                    .Location = startLocation,
                    .Length   = length,
                    .Data     = name,
                };
            }
        }
    }
}

Token Lexer::PeekToken() {
    auto copy = *this;
    return copy.NextToken();
}

char Lexer::CurrentChar() {
    if (Location.Position < Source.length()) {
        return Source[Location.Position];
    } else {
        return '\0';
    }
}

char Lexer::NextChar() {
    char current = CurrentChar();

    Location.Position++;
    Location.Column++;
    if (current == '\n') {
        Location.Line++;
        Location.Column = 1;
    }

    return current;
}

void Lexer::SkipWhitespace() {
    while (std::isspace(CurrentChar()))
        NextChar();
}
