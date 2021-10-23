from bytecode import Chunk, OpCode
from debug import disassemble_instr
from value import BoolValue, FloatValue, IntValue, NullValue
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
        assert isinstance(self.stack, list)

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

    def is_number(self, value):
        return isinstance(value, IntValue) or isinstance(value, FloatValue)

    def numbers(self, l, r):
        return self.is_number(l) and self.is_number(r)

    def span(self):
        return self.chunk.pos[self.ip]

    def runtime_error(self, message):
        print("[%s] Error: %s" % (self.span().str(), message))
        return Result.RUNTIME_ERR

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
                if self.numbers(l, r):
                    self.push(l.add(r))
                else:
                    return self.runtime_error("operands of SUM must be numbers")
            elif instruction == OpCode.DIV:
                l = self.pop()
                r = self.pop()
                if self.numbers(l, r):
                    self.push(l.div(r))
                else:
                    return self.runtime_error("operands of QUOSHUNT must be numbers")
            elif instruction == OpCode.MUL:
                l = self.pop()
                r = self.pop()
                if self.numbers(l, r):
                    self.push(l.mul(r))
                else:
                    return self.runtime_error("operands of PRODUKT must be numbers")
            elif instruction == OpCode.SUB:
                l = self.pop()
                r = self.pop()
                if self.numbers(l, r):
                    self.push(l.sub(r))
                else:
                    return self.runtime_error("operands of DIFF must be numbers")
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
            elif instruction == OpCode.PUSH_WIN:
                self.push(BoolValue(True))
            elif instruction == OpCode.PUSH_FAIL:
                self.push(BoolValue(False))
            elif instruction == OpCode.PUSH_NOOB:
                self.push(NullValue())


def interpret(source):
    chunk = compile(source, Chunk())
    if chunk is None:
        return Result.COMPILE_ERR
    vm = Vm(chunk)
    return vm.interpret()
