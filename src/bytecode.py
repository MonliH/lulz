from error import Span
from value import Value


class OpCode:
    RETURN = 0
    CONSTANT = 1
    ADD = 2
    SUB = 3
    MUL = 4
    DIV = 5
    PRINT = 6
    POP = 7

    GLOBAL_DEF = 8
    GLOBAL_GET = 9
    LOCAL_GET = 10
    LOCAL_SET = 11

    PUSH_WIN = 12
    PUSH_FAIL = 13
    PUSH_NOOB = 14

    SET_IT = 15
    GET_IT = 16

    JUMP_IF_FALSE = 17
    JUMP = 18

    EQ = 19


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
