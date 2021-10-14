from bytecode import Chunk, OpCode
from debug import disassemble_instr
from value import Value
import os


class Result:
    OK = 0
    COMPILE_ERR = 1
    RUNTIME_ERR = 2


class Vm:
    def __init__(self, chunk):
        assert isinstance(chunk, Chunk)
        self.chunk = chunk
        self.ip = 0
        self.stack = []

    def read_byte(self):
        old_ip = self.ip
        self.ip += 1
        return self.chunk.code[old_ip]

    def read_constant(self):
        return self.chunk.constants[self.read_byte()]

    def pop(self):
        popped = self.stack.pop()
        assert isinstance(popped, Value)
        return popped

    def push(self, value):
        assert isinstance(value, Value)
        self.stack.append(value)

    def interpret(self):
        while True:
            disassemble_instr(self.chunk, self.ip)
            os.write(1, "      ")
            for value in self.stack:
                os.write(1, "[%s]" % value.str())
            print()
            instruction = self.read_byte()
            if instruction == OpCode.OP_RETURN:
                print(self.pop().str())
                return Result.OK
            elif instruction == OpCode.OP_CONSTANT:
                constant = self.read_constant()
                self.push(constant)
            elif instruction == OpCode.OP_ADD:
                l = self.pop()
                r = self.pop()
                self.push(l.add(r))
