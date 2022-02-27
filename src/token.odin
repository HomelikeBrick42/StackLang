package stack

TokenKind :: enum {
	Invalid,

	// Special
	EOF,
	Integer,
	Name,

	// Seperators
	OpenParenthesis,
	CloseParenthesis,
	OpenBrace,
	CloseBrace,

	// Keywords
	Add,
	Subtract,
	Multiply,
	Divide,
	Equal,
	Assign,
	Print,
	If,
	Else,
	Drop,
	Dup,
}

Token :: struct {
	kind:           TokenKind,
	using location: SourceLocation,
	length:         uint,
	data:           union {
		string,
		u128,
	},
}
