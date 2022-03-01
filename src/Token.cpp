#include "Token.hpp"

#include <cassert>

std::string_view TokenKind_ToString(TokenKind kind) {
    switch (kind) {
        case TokenKind::Invalid:
            return "Invalid";
        case TokenKind::EndOfFile:
            return "EndOfFile";
        case TokenKind::Name:
            return "Name";
        case TokenKind::Integer:
            return "Integer";
        case TokenKind::OpenBrace:
            return "{";
        case TokenKind::CloseBrace:
            return "}";
        case TokenKind::Add:
            return "+";
        case TokenKind::Subtract:
            return "-";
        case TokenKind::Multiply:
            return "*";
        case TokenKind::Divide:
            return "/";
        case TokenKind::Modulus:
            return "%";
        case TokenKind::Equal:
            return "==";
        case TokenKind::NotEqual:
            return "!=";
        case TokenKind::LessThan:
            return "<";
        case TokenKind::GreaterThan:
            return ">";
        case TokenKind::LessThanOrEqual:
            return "<=";
        case TokenKind::GreaterThanOrEqual:
            return ">=";
        case TokenKind::Not:
            return "!";
        case TokenKind::Print:
            return "print";
        case TokenKind::If:
            return "if";
        case TokenKind::Else:
            return "else";
        case TokenKind::While:
            return "while";
        case TokenKind::Dup:
            return "dup";
        case TokenKind::Drop:
            return "drop";
    }
    assert(false);
    std::exit(-1);
}
