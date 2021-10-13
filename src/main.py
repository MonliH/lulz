import sys
from bytecode import OpCode, Chunk
import debug


def entry_point(argv):
    chunk = Chunk()
    chunk.ops.append(OpCode.OP_RETURN)
    debug.disassemble(chunk, "test chunk")
    return 0


def target(driver, *args):
    return entry_point, None


if __name__ == "__main__":
    entry_point(sys.argv)
