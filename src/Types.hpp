#pragma once

#include "Common.hpp"

enum struct Type {
    Invalid,
    Integer,
    Bool,
};

std::string_view Type_ToString(Type type);
