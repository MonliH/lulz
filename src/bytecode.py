class OpCode:
    OP_RETURN = 0
    OP_CONSTANT = 1


class Chunk:
    def __init__(self):
        self.constants = []
        self.ops = []
