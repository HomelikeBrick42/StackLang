#include "Types.hpp"

#include <cassert>

std::string_view Type_ToString(const Type& type) {
    switch (type.Kind) {
        case TypeKind::Invalid:
            return "Invalid";
        case TypeKind::Type:
            return "Type";
        case TypeKind::Integer:
            return "Integer";
        case TypeKind::Bool:
            return "Bool";
    }
    assert(false);
    std::exit(-1);
}

Type::Type(TypeKind kind)
    : Kind(kind), PointerDepth(0) {}
