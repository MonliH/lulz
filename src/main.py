import sys
from bytecode import OpCode, Chunk
import debug
from value import IntValue
from error import Span
from vm import Result, Vm, interpret


def run_file(filename):
    assert isinstance(filename, str)
    source = ""
    with open(filename, "r") as f:
        source = f.read()
    result = interpret(source)
    if result == Result.COMPILE_ERR:
        return 65
    elif result == Result.RUNTIME_ERR:
        return 70
    return 0


def entry_point(argv):
    if len(argv) == 2:
        return run_file(argv[1])
    else:
        print("Usage: lulz [path]")
        return 64

    # chunk = Chunk()
    # constant = chunk.add_constant(IntValue(2))
    # constant2 = chunk.add_constant(IntValue(1))

    # chunk.write(OpCode.OP_CONSTANT, Span(0, 0))
    # chunk.write(constant, Span(0, 1))
    # chunk.write(OpCode.OP_CONSTANT, Span(0, 0))
    # chunk.write(constant2, Span(0, 1))
    # chunk.write(OpCode.OP_CONSTANT, Span(0, 0))
    # chunk.write(constant, Span(0, 1))

    # chunk.write(OpCode.OP_ADD, Span(0, 1))
    # chunk.write(OpCode.OP_MUL, Span(0, 1))

    # chunk.write(OpCode.OP_RETURN, Span(1, 2))
    # debug.disassemble(chunk, "test chunk")

    # print("------")

    # vm = Vm(chunk)
    # vm.interpret()


def target(driver, *args):
    return entry_point, None


if __name__ == "__main__":
    entry_point(sys.argv)
