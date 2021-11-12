from bytecode import Chunk, OpCode
from debug import disassemble, disassemble_instr
from value import BoolValue, FloatValue, FuncValue, IntValue, NullValue
from compiler import compile
from rpython.rlib import rfile
import os


class Result:
    OK = 0
    COMPILE_ERR = 1
    RUNTIME_ERR = 2


class Limits:
    STACK_MAX = 256
    FRAMES_MAX = 256


class CallFrame:
    __slots__ = ["fn", "ip", "frame_start"]
    _immutable_fields_ = ["fn"]

    def __init__(self, fn, ip, frame_start):
        self.fn = fn
        self.ip = ip
        self.frame_start = frame_start


class Vm:
    def __init__(self):
        self.stack = [None] * Limits.STACK_MAX
        self.stack_top = 0

        self.frames = []
        self.frame = None

        self.globals = {}
        self.it = NullValue()

    def read_byte(self):
        old_ip = self.frame.ip
        self.frame.ip += 1
        return self.frame.fn.chunk.code[old_ip]

    def read_constant(self):
        return self.frame.fn.chunk.constants[self.read_byte()]

    def push(self, value):
        self.stack[self.stack_top] = value
        self.stack_top += 1

    def pop(self):
        self.stack_top -= 1
        return self.stack[self.stack_top]

    def is_number(self, value):
        return isinstance(value, IntValue) or isinstance(value, FloatValue)

    def numbers(self, l, r):
        return self.is_number(l) and self.is_number(r)

    def span(self):
        return self.frame.fn.chunk.pos[self.frame.ip]

    def runtime_error(self, message):
        print("[%s] Error: %s, IDIOT!" % (self.span().str(), message))
        return Result.RUNTIME_ERR

    def vton(self, value):
        """Convert a value to a number"""
        num_val = value.to_number()
        if num_val:
            return num_val
        self.runtime_error("%s is not a NUMBAR or NUMBR" % value.dbg())

    def call_value(self, callee, arg_count):
        if isinstance(callee, FuncValue):
            if arg_count != callee.arity:
                return False
            self.frames.append(CallFrame(callee, 0, self.stack_top - arg_count - 1))
            self.frame = self.frames[-1]
            return True

        self.runtime_error("Can only call FUNKSHUNS")
        return False

    def peek(self, dist):
        return self.stack[self.stack_top - 1 - dist]

    def vm_print(self):
        num = self.read_byte()

        for i in range(num):
            value = self.peek(num - i - 1)
            os.write(1, value.str())

        self.stack_top -= num

    def interpret(self):
        # stdin, _, _ = rfile.create_stdio()
        while True:
            # stdin.readline(1024)
            # disassemble_instr(self.frame.fn.chunk, self.frame.ip)
            # os.write(2, "      ")
            # for i in range(self.stack_top):
            #     os.write(2, "[%s]" % self.stack[i].str())
            # os.write(2, " IT: %s" % self.it.str())
            # os.write(2, "\n      { ")
            # for (k, v) in self.globals.items():
            #     os.write(2, "%d: %s, " % (k, v.str()))
            # os.write(2, " }\n")

            instruction = self.read_byte()
            if instruction == OpCode.RETURN:
                ret_val = self.pop()
                self.frames.pop()
                if len(self.frames) == 0:
                    # Pop off the <script> function
                    self.pop()
                    return Result.OK

                self.stack_top = self.frame.frame_start
                self.frame = self.frames[-1]
                self.push(ret_val)
            elif instruction == OpCode.CALL:
                arg_count = self.read_byte()
                if not self.call_value(self.peek(arg_count), arg_count):
                    return Result.RUNTIME_ERR
                self.frame = self.frames[-1]
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
                self.vm_print()
            elif instruction == OpCode.PRINTLN:
                self.vm_print()
                os.write(1, "\n")
            elif instruction == OpCode.POP:
                self.pop()
            elif instruction == OpCode.GLOBAL_DEF:
                idx = self.read_byte()
                self.globals[idx] = self.pop()
            elif instruction == OpCode.GLOBAL_GET:
                idx = self.read_byte()
                self.push(self.globals[idx])
            elif instruction == OpCode.LOCAL_GET:
                idx = self.read_byte()
                self.push(self.stack[self.frame.frame_start + idx])
            elif instruction == OpCode.LOCAL_SET:
                idx = self.read_byte()
                # Keep the expression on the stack
                # (i.e., assignments are expressions)
                self.stack[self.frame.frame_start + idx] = self.peek(0)
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
                    self.frame.ip += offset
            elif instruction == OpCode.JUMP:
                offset = self.read_byte()
                self.frame.ip += offset
            else:
                print("Internal Error: Unknown Instruction %s" % instruction)
                return Result.RUNTIME_ERR


def interpret(source):
    function = compile(source)
    if function is None:
        return Result.COMPILE_ERR

    vm = Vm()
    vm.push(function)
    vm.call_value(function, 0)

    return vm.interpret()
