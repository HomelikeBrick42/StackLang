package stack

import "core:unicode"
import "core:strconv"

@(private = "file")
lexer_seperators := map[rune]TokenKind {
	'(' = .OpenParenthesis,
	')' = .CloseParenthesis,
	'{' = .OpenBrace,
	'}' = .CloseBrace,
}

@(private = "file")
lexer_keywords := map[string]TokenKind {
	"+"     = .Add,
	"-"     = .Subtract,
	"*"     = .Multiply,
	"/"     = .Divide,
	"=="    = .Equal,
	"="     = .Assign,
	"print" = .Print,
	"if"    = .If,
	"else"  = .Else,
	"drop"  = .Drop,
	"dup"   = .Dup,
}

Lexer :: struct {
	using location: SourceLocation,
	source:         string,
}

Lexer_Create :: proc(filepath, source: string) -> Lexer {
	return Lexer{
		location = {filepath = filepath, position = 0, line = 1, column = 1},
		source = source,
	}
}

@(private = "file")
Lexer_CurrentRune :: proc(lexer: Lexer) -> rune {
	if lexer.position < len(lexer.source) {
		return rune(lexer.source[lexer.position])
	} else {
		return 0
	}
}

@(private = "file")
Lexer_NextRune :: proc(lexer: ^Lexer) -> rune {
	current := Lexer_CurrentRune(lexer^)
	if current == 0 do return 0

	lexer.position += 1
	lexer.column += 1
	if current == '\n' {
		lexer.line += 1
		lexer.column = 1
	}

	return current
}

@(private = "file")
Lexer_SkipWhitespace :: proc(lexer: ^Lexer) {
	for unicode.is_white_space(Lexer_CurrentRune(lexer^)) {
		Lexer_NextRune(lexer)
	}
}

Lexer_NextToken :: proc(lexer: ^Lexer) -> Token {
	Lexer_SkipWhitespace(lexer)
	start_location := lexer.location
	if Lexer_CurrentRune(lexer^) == 0 {
		return Token{
			kind = .EOF,
			location = start_location,
			length = lexer.position - start_location.position,
			data = nil,
		}
	}

	for
	    Lexer_CurrentRune(lexer^) != 0 && !unicode.is_white_space(
		    Lexer_CurrentRune(lexer^),
	    ) {
		if _, found := lexer_seperators[Lexer_CurrentRune(lexer^)]; found {
			break
		}
		Lexer_NextRune(lexer)
	}

	if lexer.location == start_location {
		if kind, found := lexer_seperators[Lexer_CurrentRune(lexer^)]; found {
			Lexer_NextRune(lexer)
			return Token{
				kind = kind,
				location = start_location,
				length = lexer.position - start_location.position,
				data = nil,
			}
		} else {
			unreachable()
		}
	} else {
		text := lexer.source[start_location.position:lexer.position]
		if value, ok := strconv.parse_u128_maybe_prefixed(text); ok {
			return Token{
				kind = .Integer,
				location = start_location,
				length = lexer.position - start_location.position,
				data = value,
			}
		} else if kind, found := lexer_keywords[text]; found {
			return Token{
				kind = kind,
				location = start_location,
				length = lexer.position - start_location.position,
				data = nil,
			}
		} else {
			return Token{
				kind = .Name,
				location = start_location,
				length = lexer.position - start_location.position,
				data = text,
			}
		}
	}
}

Lexer_PeekToken :: proc(lexer: Lexer) -> Token {
	copy := lexer
	return Lexer_NextToken(&copy)
}
