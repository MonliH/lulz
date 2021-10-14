import sys
from bytecode import OpCode, Chunk
import debug
from value import IntValue
from error import Span
from vm import Vm

def entry_point(argv):
    chunk = Chunk()
    constant = chunk.add_constant(IntValue(2))
    constant2 = chunk.add_constant(IntValue(1))

    chunk.write(OpCode.OP_CONSTANT, Span(0, 0))
    chunk.write(constant, Span(0, 1))
    chunk.write(OpCode.OP_CONSTANT, Span(0, 0))
    chunk.write(constant2, Span(0, 1))
    chunk.write(OpCode.OP_CONSTANT, Span(0, 0))
    chunk.write(constant, Span(0, 1))

    chunk.write(OpCode.OP_ADD, Span(0, 1))
    chunk.write(OpCode.OP_MUL, Span(0, 1))

    chunk.write(OpCode.OP_RETURN, Span(1, 2))
    debug.disassemble(chunk, "test chunk")

    print("------")

    vm = Vm(chunk)
    vm.interpret()

    return 0


def target(driver, *args):
    return entry_point, None


if __name__ == "__main__":
    entry_point(sys.argv)
