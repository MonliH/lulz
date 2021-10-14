class OpCode:
    OP_RETURN = 0
    OP_CONSTANT = 1


class Chunk:
    def __init__(self, constants=[], code=[], pos=[]):
        self.constants = constants
        self.code = code
        self.pos = pos

    def add_constant(self, value):
        self.constants.append(value)
        return len(self.constants) - 1

    def write(self, b, span):
        self.code.append(b)
        self.pos.append(span)
