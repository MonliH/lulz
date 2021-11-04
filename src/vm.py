from bytecode import Chunk, OpCode
from debug import disassemble, disassemble_instr
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
        self.it = NullValue()

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

    def vton(self, value):
        """Convert a value to a number"""
        num_val = value.to_number()
        if num_val:
            return num_val
        self.runtime_error("%s is not a NUMBAR or NUMBR" % value.dbg())

    def interpret(self):
        while True:
            disassemble_instr(self.chunk, self.ip)
            os.write(2, "      ")
            for value in self.stack:
                os.write(2, "[%s]" % value.str())
            os.write(2, " IT: %s" % self.it.str())
            os.write(2, "\n      { ")
            for (k, v) in self.globals.items():
                os.write(2, "%d: %s, " % (k, v.str()))
            os.write(2, " }\n")

            instruction = self.read_byte()
            if instruction == OpCode.RETURN:
                return Result.OK
            elif instruction == OpCode.CONSTANT:
                constant = self.read_constant()
                self.push(constant)
            elif instruction == OpCode.ADD:
                r = self.vton(self.pop())
                l = self.vton(self.pop())
                self.push(l.add(r))
            elif instruction == OpCode.DIV:
                r = self.vton(self.pop())
                l = self.vton(self.pop())
                self.push(l.div(r))
            elif instruction == OpCode.MUL:
                r = self.vton(self.pop())
                l = self.vton(self.pop())
                self.push(l.mul(r))
            elif instruction == OpCode.SUB:
                r = self.vton(self.pop())
                l = self.vton(self.pop())
                self.push(l.sub(r))
            elif instruction == OpCode.GT:
                r = self.vton(self.pop())
                l = self.vton(self.pop())
                self.push(l.gt(r))
            elif instruction == OpCode.GTE:
                r = self.vton(self.pop())
                l = self.vton(self.pop())
                self.push(l.gte(r))
            elif instruction == OpCode.LT:
                r = self.vton(self.pop())
                l = self.vton(self.pop())
                self.push(l.lt(r))
            elif instruction == OpCode.LTE:
                r = self.vton(self.pop())
                l = self.vton(self.pop())
                self.push(l.lte(r))
            elif instruction == OpCode.MIN:
                r = self.vton(self.pop())
                l = self.vton(self.pop())
                self.push(l.min(r))
            elif instruction == OpCode.MAX:
                r = self.vton(self.pop())
                l = self.vton(self.pop())
                self.push(l.max(r))
            elif instruction == OpCode.PRINT:
                num = self.read_byte()

                for i in range(num):
                    value = self.stack[len(self.stack) - num + i]
                    os.write(1, value.str())

                os.write(1, "\n")
            elif instruction == OpCode.POP:
                self.pop()
            elif instruction == OpCode.GLOBAL_DEF:
                idx = self.read_byte()
                self.globals[idx] = self.stack[len(self.stack) - 1]
            elif instruction == OpCode.GLOBAL_GET:
                idx = self.read_byte()
                self.push(self.globals[idx])
            elif instruction == OpCode.LOCAL_GET:
                idx = self.read_byte()
                self.push(self.stack[idx])
            elif instruction == OpCode.LOCAL_SET:
                idx = self.read_byte()
                # Keep the expression on the stack
                # (i.e., assignments are expressions)
                self.stack[idx] = self.stack[len(self.stack) - 1]
            elif instruction == OpCode.PUSH_WIN:
                self.push(BoolValue(True))
            elif instruction == OpCode.PUSH_FAIL:
                self.push(BoolValue(False))
            elif instruction == OpCode.PUSH_NOOB:
                self.push(NullValue())
            elif instruction == OpCode.SET_IT:
                self.it = self.pop()
            elif instruction == OpCode.GET_IT:
                self.push(self.it)
            elif instruction == OpCode.EQ:
                r = self.pop()
                l = self.pop()
                self.push(l.eq(r))
            elif instruction == OpCode.JUMP_IF_FALSE:
                condition = self.it
                offset = self.read_byte()
                if not condition.is_truthy():
                    self.ip += offset
            elif instruction == OpCode.JUMP:
                offset = self.read_byte()
                self.ip += offset


def interpret(source):
    chunk = compile(source, Chunk())
    if chunk is None:
        return Result.COMPILE_ERR
    disassemble(chunk, "Main")
    vm = Vm(chunk)
    return vm.interpret()
