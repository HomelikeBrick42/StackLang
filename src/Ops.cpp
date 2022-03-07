#include "Ops.hpp"

#include <cassert>

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
        case OpKind::TypePush:
            return "TypePush";
        case OpKind::TypeDup:
            return "TypeDup";
        case OpKind::TypeDrop:
            return "TypeDrop";
        case OpKind::TypePointerTo:
            return "TypePointerTo";
        case OpKind::TypeEqual:
            return "TypeEqual";
        case OpKind::TypePrint:
            return "TypePrint";
    }
    assert(false);
    std::exit(-1);
}
