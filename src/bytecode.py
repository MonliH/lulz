from error import Span
from value import Value


class OpCode:
    OP_RETURN = 0
    OP_CONSTANT = 1
    OP_ADD = 2
    OP_SUB = 3
    OP_MUL = 4
    OP_DIV = 5


class Chunk:
    def __init__(self, constants=[], code=[], pos=[]):
        self.constants = constants
        self.code = code
        self.pos = pos

    def add_constant(self, value):
        assert isinstance(value, Value)
        self.constants.append(value)
        return len(self.constants) - 1

    def write(self, b, span):
        assert isinstance(span, Span)
        self.code.append(b)
        self.pos.append(span)
