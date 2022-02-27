package stack

import "core:mem"
import "core:fmt"
import "core:io"
import "core:os"

Main :: proc() -> Maybe(CompileError) {
	filepath := "test.stack"
	source := `
		1 2 3 * +
		if dup 7 == {
			drop
			10
		}
		print
	`
	ops, err := CompileOps(filepath, source)
	defer delete(ops)
	err or_return

	when false {
		for op, i in ops {
			fmt.printf("%i: %v\n", i, op)
		}
		fmt.println()
	}

	delete(ExecuteOps(ops[:]))

	return nil
}

main :: proc() {
	formatters: map[typeid]fmt.User_Formatter
	fmt.set_user_formatters(&formatters)
	fmt.register_user_formatter(
		typeid_of(SourceLocation),
		proc(fi: ^fmt.Info, arg: any, verb: rune) -> bool {
			location := arg.(SourceLocation)
			io.write_string(fi.writer, location.filepath)
			io.write_rune(fi.writer, ':')
			io.write_uint(fi.writer, location.line)
			io.write_rune(fi.writer, ':')
			io.write_uint(fi.writer, location.column)
			return true
		},
	)
	fmt.register_user_formatter(
		typeid_of(TokenKind),
		proc(fi: ^fmt.Info, arg: any, verb: rune) -> bool {
			kind := arg.(TokenKind)
			switch kind {
			case .Invalid:
				io.write_string(fi.writer, "Invalid")
			case .EOF:
				io.write_string(fi.writer, "EOF")
			case .Integer:
				io.write_string(fi.writer, "Integer")
			case .Name:
				io.write_string(fi.writer, "Name")
			case .OpenParenthesis:
				io.write_string(fi.writer, "(")
			case .CloseParenthesis:
				io.write_string(fi.writer, ")")
			case .OpenBrace:
				io.write_string(fi.writer, "{")
			case .CloseBrace:
				io.write_string(fi.writer, "}")
			case .Add:
				io.write_string(fi.writer, "+")
			case .Subtract:
				io.write_string(fi.writer, "-")
			case .Multiply:
				io.write_string(fi.writer, "*")
			case .Divide:
				io.write_string(fi.writer, "/")
			case .Equal:
				io.write_string(fi.writer, "==")
			case .Assign:
				io.write_string(fi.writer, "=")
			case .Print:
				io.write_string(fi.writer, "print")
			case .If:
				io.write_string(fi.writer, "if")
			case .Else:
				io.write_string(fi.writer, "else")
			case .Drop:
				io.write_string(fi.writer, "drop")
			case .Dup:
				io.write_string(fi.writer, "dup")
			}
			return true
		},
	)

	when ODIN_DEBUG {
		track: mem.Tracking_Allocator
		mem.tracking_allocator_init(&track, context.allocator)
		context.allocator = mem.tracking_allocator(&track)
	}

	error, was_error := Main().?
	if was_error {
		fmt.printf("%v: Error: %s\n", error.location, error.message)
		delete(error.message)
	}

	when ODIN_DEBUG {
		for _, v in track.allocation_map {
			fmt.printf("%v leaked %v bytes", v.location, v.size)
		}
		for bf in track.bad_free_array {
			fmt.printf("%v allocation %p was freed badly", bf.location, bf.memory)
		}
	}

	os.exit(1 if was_error else 0)
}
