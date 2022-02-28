#pragma once

#include <string_view>

struct SourceLocation {
    std::string_view Filepath{};
    size_t Position{};
    size_t Line{};
    size_t Column{};
};
