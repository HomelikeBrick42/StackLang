#pragma once

#include "Common.hpp"
#include "Ops.hpp"

#include <vector>

std::vector<Op> CompileOps(std::string_view filepath, std::string_view source);
