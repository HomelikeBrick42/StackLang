#pragma once

#include "Common.hpp"

enum struct TypeKind {
    Invalid,
    Type,
    Integer,
    Bool,
};

struct Type {
    TypeKind Kind = TypeKind::Invalid;
    size_t PointerDepth = 0;

    Type() = default;
    Type(TypeKind kind);

    bool operator==(const Type& other) const {
        return this->Kind == other.Kind && this->PointerDepth == other.PointerDepth;
    }

    bool operator!=(const Type& other) const {
        return !(*this == other);
    }
};

std::string_view Type_ToString(const Type& type);
