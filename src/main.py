import sys
import os
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
        os.write(2, "Usage: lulz [path]\n")
        return 64


def target(driver, *args):
    return entry_point, None


if __name__ == "__main__":
    entry_point(sys.argv)
