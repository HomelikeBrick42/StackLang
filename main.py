from enum import Enum, auto
import sys


class Op(Enum):
    Push = auto()
    Pop = auto()
    Add = auto()
    Sub = auto()
    Not = auto()
    Eq = auto()
    Dup = auto()
    Dump = auto()
    If = auto()
    Else = auto()
    End = auto()
    Count = auto()


def simulate_push(program, stack, op):
    stack.append(op[1])


def simulate_pop(program, stack, op):
    stack.pop()


def simulate_add(program, stack, op):
    b = stack.pop()
    a = stack.pop()
    stack.append(a + b)


def simulate_sub(program, stack, op):
    b = stack.pop()
    a = stack.pop()
    stack.append(a - b)


def simulate_not(program, stack, op):
    value = stack.pop()
    stack.append(int(not value))


def simulate_eq(program, stack, op):
    b = stack.pop()
    a = stack.pop()
    stack.append(int(a == b))


def simulate_dup(program, stack, op):
    value = stack.pop()
    stack.append(value)
    stack.append(value)


def simulate_dump(program, stack, op):
    value = stack.pop()
    print(value)


def simulate_if(program, stack, op):
    condition = stack.pop()
    if not condition:
        depth = 1
        while depth > 0:
            operation = next(program)
            if operation[0] == Op.If:
                depth += 1
            elif operation[0] == Op.Else and depth == 1:
                break
            elif operation[0] == Op.End:
                depth -= 1
    else:
        while True:
            operation = next(program)
            simulate_operations[operation[0]](program, stack, operation)
            if operation[0] == Op.Else:
                depth = 0
                while True:
                    operation = next(program)
                    if operation[0] == Op.If:
                        depth += 1
                    elif operation[0] == Op.End:
                        if depth == 0:
                            break
                        depth -= 1
                break


def simulate_else(program, stack, op):
    pass


def simulate_end(program, stack, op):
    pass


simulate_operations = {
    Op.Push: simulate_push,
    Op.Pop: simulate_pop,
    Op.Add: simulate_add,
    Op.Sub: simulate_sub,
    Op.Not: simulate_not,
    Op.Eq: simulate_eq,
    Op.Dup: simulate_dup,
    Op.Dump: simulate_dump,
    Op.If: simulate_if,
    Op.Else: simulate_else,
    Op.End: simulate_end,
}


def simulate_program(program):
    stack = []
    for op in program:
        simulate_operations[op[0]](program, stack, op)


op_strings = {
    "pop": (Op.Pop,),
    "+": (Op.Add,),
    "-": (Op.Sub,),
    "not": (Op.Not,),
    "=": (Op.Eq,),
    "dup": (Op.Dup,),
    ".": (Op.Dump,),
    "if": (Op.If,),
    "else": (Op.Else,),
    "end": (Op.End,),
}


def op_from_token(token):
    if token in op_strings:
        return op_strings[token]
    else:
        return (Op.Push, int(token))


def program_from_str(source):
    return (op_from_token(token) for token in source.split())


def main():
    if len(sys.argv) != 2:
        print("Usage: %s <file>" % sys.argv[0])
        exit(1)

    try:
        with open(sys.argv[1], "r") as file:
            program = program_from_str(file.read())
            simulate_program(program)
    except FileNotFoundError:
        print("'%s' does not exist!" % sys.argv[1])


if __name__ == "__main__":
    main()
