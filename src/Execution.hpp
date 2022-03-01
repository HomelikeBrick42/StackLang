#pragma once

#include "Common.hpp"
#include "Ops.hpp"

#include <span>
#include <vector>

using Value = std::variant<long long, bool>;

std::vector<Value> ExecuteOps(std::span<Op> ops);
