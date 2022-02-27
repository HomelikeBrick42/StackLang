package stack

Op :: struct {
	using location: SourceLocation,
	data:           union {
		ExitOp,
		JumpOp,
		JumpFalseOp,

		// Integer
		IntegerPushOp,
		IntegerDropOp,
		IntegerDupOp,
		IntegerAddOp,
		IntegerSubtractOp,
		IntegerMultiplyOp,
		IntegerDivideOp,
		IntegerEqualOp,
		IntegerPrintOp,

		// Bool
		BoolPushOp,
		BoolDropOp,
		BoolDupOp,
		BoolEqualOp,
		BoolPrintOp,
	},
}

ExitOp :: struct {}
JumpOp :: struct {
	relative_ip: int,
}
JumpFalseOp :: struct {
	relative_ip: int,
}

// Integer
IntegerPushOp :: struct {
	value: i64,
}
IntegerDropOp :: struct {}
IntegerDupOp :: struct {}
IntegerAddOp :: struct {}
IntegerSubtractOp :: struct {}
IntegerMultiplyOp :: struct {}
IntegerDivideOp :: struct {}
IntegerEqualOp :: struct {}
IntegerPrintOp :: struct {}

// Bool
BoolPushOp :: struct {
	value: bool,
}
BoolDropOp :: struct {}
BoolDupOp :: struct {}
BoolEqualOp :: struct {}
BoolPrintOp :: struct {}
