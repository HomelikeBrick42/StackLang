#include "Execution.hpp"

#include <cassert>

std::vector<Value> ExecuteOps(std::span<Op> ops) {
    std::vector<Value> stack{};

    size_t ip = 0;
    while (true) {
        auto& op = ops[ip];
#if 0
        printf("IP: %llu\n", ip);
        printf("STACK: [");
        bool first = true;
        for (auto& value : stack) {
            if (first) {
                first = false;
            } else {
                printf(", ");
            }
            if (std::holds_alternative<long long>(value)) {
                printf("%lld", std::get<long long>(value));
            } else {
                printf("%s", std::get<bool>(value) ? "true" : "false");
            }
        }
        printf("]\n");
#endif
        switch (op.Kind) {
            case OpKind::Invalid: {
                assert(false);
                std::exit(-1);
            } break;

            case OpKind::Exit: {
                goto EndLoop;
            } break;

            case OpKind::Jump: {
                ip = static_cast<size_t>(static_cast<long long>(ip) + std::get<long long>(op.Data));
                continue;
            } break;

            case OpKind::JumpFalse: {
                bool condition = std::get<bool>(stack.back());
                stack.pop_back();
                if (!condition) {
                    ip = static_cast<size_t>(static_cast<long long>(ip) + std::get<long long>(op.Data));
                    continue;
                }
            } break;

            case OpKind::IntegerPush: {
                stack.emplace_back(std::get<long long>(op.Data));
            } break;

            case OpKind::IntegerDup: {
                auto value = std::get<long long>(stack.back());
                stack.emplace_back(value);
            } break;

            case OpKind::IntegerDrop: {
                (void)std::get<long long>(stack.back());
                stack.pop_back();
            } break;

            case OpKind::IntegerAdd: {
                auto b = std::get<long long>(stack.back());
                stack.pop_back();
                auto a = std::get<long long>(stack.back());
                stack.pop_back();
                stack.emplace_back(a + b);
            } break;

            case OpKind::IntegerSubtract: {
                auto b = std::get<long long>(stack.back());
                stack.pop_back();
                auto a = std::get<long long>(stack.back());
                stack.pop_back();
                stack.emplace_back(a - b);
            } break;

            case OpKind::IntegerMultiply: {
                auto b = std::get<long long>(stack.back());
                stack.pop_back();
                auto a = std::get<long long>(stack.back());
                stack.pop_back();
                stack.emplace_back(a * b);
            } break;

            case OpKind::IntegerDivide: {
                auto b = std::get<long long>(stack.back());
                stack.pop_back();
                auto a = std::get<long long>(stack.back());
                stack.pop_back();
                stack.emplace_back(a / b);
            } break;

            case OpKind::IntegerModulus: {
                auto b = std::get<long long>(stack.back());
                stack.pop_back();
                auto a = std::get<long long>(stack.back());
                stack.pop_back();
                stack.emplace_back(a % b);
            } break;

            case OpKind::IntegerLessThan: {
                auto b = std::get<long long>(stack.back());
                stack.pop_back();
                auto a = std::get<long long>(stack.back());
                stack.pop_back();
                stack.emplace_back(a < b);
            } break;

            case OpKind::IntegerGreaterThan: {
                auto b = std::get<long long>(stack.back());
                stack.pop_back();
                auto a = std::get<long long>(stack.back());
                stack.pop_back();
                stack.emplace_back(a > b);
            } break;

            case OpKind::IntegerEqual: {
                auto b = std::get<long long>(stack.back());
                stack.pop_back();
                auto a = std::get<long long>(stack.back());
                stack.pop_back();
                stack.emplace_back(a == b);
            } break;

            case OpKind::IntegerPrint: {
                auto value = std::get<long long>(stack.back());
                stack.pop_back();
                printf("%lld\n", value);
            } break;

            case OpKind::BoolPush: {
                stack.emplace_back(std::get<bool>(op.Data));
            } break;

            case OpKind::BoolDup: {
                auto value = std::get<bool>(stack.back());
                stack.emplace_back(value);
            } break;

            case OpKind::BoolDrop: {
                (void)std::get<bool>(stack.back());
                stack.pop_back();
            } break;

            case OpKind::BoolNot: {
                auto value = std::get<bool>(stack.back());
                stack.pop_back();
                stack.emplace_back(!value);
            } break;

            case OpKind::BoolEqual: {
                auto b = std::get<bool>(stack.back());
                stack.pop_back();
                auto a = std::get<bool>(stack.back());
                stack.pop_back();
                stack.emplace_back(a == b);
            } break;

            case OpKind::BoolPrint: {
                auto value = std::get<bool>(stack.back());
                stack.pop_back();
                printf("%s\n", value ? "true" : "false");
            } break;
        }
        ip++;
    }
EndLoop:

    return stack;
}
