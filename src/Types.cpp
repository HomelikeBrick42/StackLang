#include "Types.hpp"

#include <cassert>

std::string_view Type_ToString(Type type) {
    switch (type) {
        case Type::Invalid:
            return "Invalid";
        case Type::Integer:
            return "Integer";
        case Type::Bool:
            return "Bool";
    }
    assert(false);
    std::exit(-1);
}
