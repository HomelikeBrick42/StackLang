package stack

Type :: union {
	IntegerType,
	BoolType,
}

TypesEqual :: proc(a: Type, b: Type) -> bool {
	ok: bool
	if _, ok = a.(IntegerType); ok {
		_, ok = b.(IntegerType)
		return ok
	}
	if _, ok = a.(BoolType); ok {
		_, ok = b.(BoolType)
		return ok
	}
	return false
}

IntegerType :: struct {}

BoolType :: struct {}
