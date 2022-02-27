package stack

SourceLocation :: struct {
	filepath: string,
	position: uint,
	line:     uint,
	column:   uint,
}

CompileError :: struct {
	using location: SourceLocation,
	message:        string,
}
