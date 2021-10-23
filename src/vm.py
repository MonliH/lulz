from bytecode import Chunk, OpCode
from debug import disassemble_instr
from value import Value
from compiler import compile
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

        self.globals = {}

    def read_byte(self):
        old_ip = self.ip
        self.ip += 1
        return self.chunk.code[old_ip]

    def read_constant(self):
        return self.chunk.constants[self.read_byte()]

    def pop(self):
        popped = self.stack.pop()
        return popped

    def push(self, value):
        self.stack.append(value)

    def interpret(self):
        while True:
            disassemble_instr(self.chunk, self.ip)
            os.write(1, "      ")
            for value in self.stack:
                os.write(1, "[%s]" % value.str())
            os.write(1, "\n      { ")
            for (k, v) in self.globals.items():
                os.write(1, "%d: %s, " % (k, v.str()))
            os.write(1, " }\n")

            instruction = self.read_byte()
            if instruction == OpCode.RETURN:
                return Result.OK
            elif instruction == OpCode.CONSTANT:
                constant = self.read_constant()
                self.push(constant)
            elif instruction == OpCode.ADD:
                l = self.pop()
                r = self.pop()
                self.push(l.add(r))
            elif instruction == OpCode.DIV:
                l = self.pop()
                r = self.pop()
                self.push(l.div(r))
            elif instruction == OpCode.MUL:
                l = self.pop()
                r = self.pop()
                self.push(l.mul(r))
            elif instruction == OpCode.SUB:
                l = self.pop()
                r = self.pop()
                self.push(l.sub(r))
            elif instruction == OpCode.PRINT:
                value = self.pop()
                print(value.str())
            elif instruction == OpCode.POP:
                self.pop()
            elif instruction == OpCode.GLOBAL_DEF:
                expr = self.pop()
                idx = self.read_byte()
                self.globals[idx] = expr
            elif instruction == OpCode.GLOBAL_GET:
                idx = self.read_byte()
                self.push(self.globals[idx])
            elif instruction == OpCode.LOCAL_GET:
                idx = self.read_byte()
                self.push(self.stack[idx])
            elif instruction == OpCode.LOCAL_SET:
                idx = self.read_byte()
                self.stack[idx] = self.stack[len(self.stack) - 1]


def interpret(source):
    chunk = Chunk()
    compile(source, chunk)
    vm = Vm(chunk)
    return vm.interpret()
