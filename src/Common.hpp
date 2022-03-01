#pragma once

#include <string_view>

#define STR_FORMAT(str) static_cast<int>((str).size()), (str).data()

struct SourceLocation {
    std::string_view Filepath{};
    size_t Position{};
    size_t Line{};
    size_t Column{};
};
