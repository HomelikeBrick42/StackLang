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
		IntegerModulusOp,
		IntegerLessThanOp,
		IntegerGreaterThanOp,
		IntegerEqualOp,
		IntegerPrintOp,

		// Bool
		BoolPushOp,
		BoolDropOp,
		BoolDupOp,
		BoolNotOp,
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
IntegerModulusOp :: struct {}
IntegerLessThanOp :: struct {}
IntegerGreaterThanOp :: struct {}
IntegerEqualOp :: struct {}
IntegerPrintOp :: struct {}

// Bool
BoolPushOp :: struct {
	value: bool,
}
BoolDropOp :: struct {}
BoolDupOp :: struct {}
BoolNotOp :: struct {}
BoolEqualOp :: struct {}
BoolPrintOp :: struct {}
