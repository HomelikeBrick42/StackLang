#pragma once

#include "Common.hpp"

#include <variant>

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

std::string_view OpKind_ToString(OpKind kind);

struct Op {
    OpKind Kind = OpKind::Invalid;
    std::variant<std::monostate, long long, bool> Data{};
};
