#pragma once

#include "Common.hpp"
#include "Ops.hpp"
#include "Types.hpp"

#include <span>
#include <vector>

using Value = std::variant<long long, bool, Type>;

std::vector<Value> ExecuteOps(std::span<Op> ops);
