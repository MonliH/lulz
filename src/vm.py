from bytecode import Chunk, OpCode
from debug import disassemble, disassemble_instr
from value import BoolValue, FloatValue, FuncValue, IntValue, NullValue, StrValue, Value
from compiler import compile
import os
from rpython.rlib import jit
from rpython.rlib.jit import JitDriver, hint


class Result:
    OK = 0
    COMPILE_ERR = 1
    RUNTIME_ERR = 2


class Limits:
    STACK_MAX = 2048
    FRAMES_MAX = 256


NOOB = NullValue()

jitdriver = JitDriver(
    greens=["ip", "fn"], reds=["self"], virtualizables=["self"], is_recursive=True
)


def jitpolicy(driver):
    from rpython.jit.codewriter.policy import JitPolicy

    return JitPolicy()


class CallFrame:
    __slots__ = ["fn", "ip", "frame_start"]
    _immutable_fields_ = ["fn"]

    def __init__(self, fn, ip, frame_start):
        self.fn = fn
        self.ip = ip
        self.frame_start = frame_start


class Cell(Value):
    def __init__(self, value):
        self.value = value


class Globals:
    def __init__(self):
        self.globals = {}
        self.version = 0

    def get(self, name):
        version = self.version
        jit.promote(version)
        val = self._get(name, version)
        if isinstance(val, Cell):
            return val.value
        else:
            return val

    @jit.purefunction
    def _get(self, name, version):
        return self.globals.get(name)

    def assign(self, name, val):
        oldval = self._get(name, jit.promote(self.version))
        if oldval is None:
            self.globals[name] = val
            self.version += 1
        else:
            if isinstance(oldval, Cell):
                oldval.value = val
            else:
                self.globals[name] = Cell(val)
                self.version += 1
        self.globals[name] = val
        self.version += 1


class Vm:
    _virtualizable_ = ["stack[*]", "frames[*]", "frame"]

    def __init__(self):
        self = hint(self, access_directly=True, fresh_virtualizable=True)
        self.stack = [None] * Limits.STACK_MAX
        self.stack_top = 0

        self.frames = [None] * Limits.FRAMES_MAX
        self.frames_top = 0

        self.frame = None

        self.globals = Globals()
        self.it = NullValue()

    def read_byte(self):
        old_ip = self.frame.ip
        self.frame.ip += 1
        return self.frame.fn.chunk.code[old_ip]

    def read_constant(self):
        return self.frame.fn.chunk.constants[self.read_byte()]

    def push(self, value):
        idx = self.stack_top
        assert idx >= 0
        self.stack[idx] = value
        self.stack_top += 1

    def pop(self):
        self.stack_top -= 1
        idx = self.stack_top
        assert idx >= 0
        return self.stack[idx]

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
        if isinstance(value, IntValue) or isinstance(value, FloatValue):
            return value
        if isinstance(value, BoolValue):
            return IntValue(int(value.bool_val))
        if isinstance(value, StrValue):
            if "." in value.str_val:
                return FloatValue(float(value.str_val))
            else:
                return IntValue(int(value.str_val))
        self.runtime_error("%s is not a NUMBAR or NUMBR" % value.dbg())

    def add(self, left, right):
        if isinstance(left, StrValue) and isinstance(right, StrValue):
            return StrValue(left.str_val + right.str_val)

        left = self.vton(left)
        right = self.vton(right)

        if isinstance(left, IntValue) and isinstance(right, IntValue):
            return IntValue(left.int_val + right.int_val)
        if isinstance(left, FloatValue):
            if isinstance(right, FloatValue):
                return FloatValue(left.float_val + right.float_val)
            if isinstance(right, IntValue):
                return FloatValue(left.float_val + float(right.int_val))
        assert False

    def sub(self, left, right):
        left = self.vton(left)
        right = self.vton(right)

        if isinstance(left, IntValue) and isinstance(right, IntValue):
            return IntValue(left.int_val - right.int_val)
        if isinstance(left, FloatValue):
            if isinstance(right, FloatValue):
                return FloatValue(left.float_val - right.float_val)
            if isinstance(right, IntValue):
                return FloatValue(left.float_val - float(right.int_val))
        assert False

    def mul(self, left, right):
        left = self.vton(left)
        right = self.vton(right)

        if isinstance(left, IntValue) and isinstance(right, IntValue):
            return IntValue(left.int_val * right.int_val)
        if isinstance(left, FloatValue):
            if isinstance(right, FloatValue):
                return FloatValue(left.float_val * right.float_val)
            if isinstance(right, IntValue):
                return FloatValue(left.float_val * float(right.int_val))
        assert False

    def div(self, left, right):
        left = self.vton(left)
        right = self.vton(right)

        if isinstance(left, IntValue) and isinstance(right, IntValue):
            return FloatValue(left.int_val / right.int_val)
        if isinstance(left, FloatValue):
            if isinstance(right, FloatValue):
                return FloatValue(left.float_val / right.float_val)
            if isinstance(right, IntValue):
                return FloatValue(left.float_val / float(right.int_val))
        assert False

    def lt(self, left, right):
        left = self.vton(left)
        right = self.vton(right)

        if isinstance(left, IntValue) and isinstance(right, IntValue):
            return BoolValue(left.int_val < right.int_val)
        if isinstance(left, FloatValue):
            if isinstance(right, FloatValue):
                return BoolValue(left.float_val < right.float_val)
            if isinstance(right, IntValue):
                return BoolValue(left.float_val < float(right.int_val))
        assert False

    def gte(self, left, right):
        left = self.vton(left)
        right = self.vton(right)

        if isinstance(left, IntValue) and isinstance(right, IntValue):
            return BoolValue(left.int_val >= right.int_val)
        if isinstance(left, FloatValue):
            if isinstance(right, FloatValue):
                return BoolValue(left.float_val >= right.float_val)
            if isinstance(right, IntValue):
                return BoolValue(left.float_val >= float(right.int_val))
        assert False

    def gt(self, left, right):
        left = self.vton(left)
        right = self.vton(right)

        if isinstance(left, IntValue) and isinstance(right, IntValue):
            return BoolValue(left.int_val > right.int_val)
        if isinstance(left, FloatValue):
            if isinstance(right, FloatValue):
                return BoolValue(left.float_val > right.float_val)
            if isinstance(right, IntValue):
                return BoolValue(left.float_val > float(right.int_val))
        assert False

    def lte(self, left, right):
        left = self.vton(left)
        right = self.vton(right)

        if isinstance(left, IntValue) and isinstance(right, IntValue):
            return BoolValue(left.int_val <= right.int_val)
        if isinstance(left, FloatValue):
            if isinstance(right, FloatValue):
                return BoolValue(left.float_val <= right.float_val)
            if isinstance(right, IntValue):
                return BoolValue(left.float_val <= float(right.int_val))
        assert False

    def is_truthy(self, val):
        if isinstance(val, BoolValue):
            return val.bool_val
        if isinstance(val, IntValue):
            return bool(val.int_val)
        if isinstance(val, FloatValue):
            return bool(val.float_val)
        if isinstance(val, StrValue):
            return bool(val.str_val)
        if isinstance(val, FuncValue):
            return True
        return False

    def call_value(self, callee, arg_count):
        if not isinstance(callee, FuncValue):
            self.runtime_error("Can only call FUNKSHUNS")
            return False

        jit.promote(callee)
        if arg_count != callee.arity:
            return False
        idx = self.frames_top
        assert idx >= 0
        self.frames[idx] = CallFrame(callee, 0, self.stack_top - arg_count - 1)
        self.frames_top += 1
        idx = self.frames_top - 1
        assert idx >= 0
        self.frame = self.frames[idx]
        return True

    def peek(self, dist):
        idx = self.stack_top - 1 - dist
        assert idx >= 0
        return self.stack[idx]

    @jit.unroll_safe
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
            jitdriver.jit_merge_point(ip=self.frame.ip, fn=self.frame.fn, self=self)

            instruction = self.read_byte()
            if instruction == OpCode.CONSTANT:
                constant = self.read_constant()
                self.push(constant)
            elif instruction == OpCode.JUMP_IF_FALSE:
                condition = self.it
                offset = self.read_byte()
                if not self.is_truthy(condition):
                    self.frame.ip += offset
            elif instruction == OpCode.JUMP:
                offset = self.read_byte()
                self.frame.ip += offset
            elif instruction == OpCode.CALL:
                arg_count = self.read_byte()
                if not self.call_value(self.peek(arg_count), arg_count):
                    return Result.RUNTIME_ERR
                idx = self.frames_top - 1
                assert idx >= 0
                self.frame = self.frames[idx]
                self.it = NOOB
            elif instruction == OpCode.GLOBAL_GET:
                idx = self.read_byte()
                self.push(self.globals.get(idx))
            elif instruction == OpCode.LOCAL_GET:
                idx = self.read_byte() + self.frame.frame_start
                assert idx >= 0
                self.push(self.stack[idx])
            elif instruction == OpCode.SET_IT:
                self.it = self.pop()
            elif instruction == OpCode.GET_IT:
                self.push(self.it)
            elif instruction == OpCode.LOOP:
                offset = self.read_byte()
                self.frame.ip -= offset
            elif instruction == OpCode.ADD:
                r = self.pop()
                l = self.pop()
                self.push(self.add(l, r))
            elif instruction == OpCode.DIV:
                r = self.pop()
                l = self.pop()
                self.push(self.div(l, r))
            elif instruction == OpCode.MUL:
                r = self.pop()
                l = self.pop()
                self.push(self.mul(l, r))
            elif instruction == OpCode.SUB:
                r = self.pop()
                l = self.pop()
                self.push(self.sub(l, r))
            elif instruction == OpCode.GT:
                r = self.vton(self.pop())
                l = self.vton(self.pop())
                self.push(self.gt(l, r))
            elif instruction == OpCode.GTE:
                r = self.vton(self.pop())
                l = self.vton(self.pop())
                self.push(self.gte(l, r))
            elif instruction == OpCode.LT:
                r = self.vton(self.pop())
                l = self.vton(self.pop())
                self.push(self.lt(l, r))
            elif instruction == OpCode.LTE:
                r = self.pop()
                l = self.pop()
                self.push(self.lte(l, r))
            # elif instruction == OpCode.MIN:
            #     r = self.vton(self.pop())
            #     l = self.vton(self.pop())
            #     self.push(l.min(r))
            # elif instruction == OpCode.MAX:
            #     r = self.vton(self.pop())
            #     l = self.vton(self.pop())
            #     self.push(l.max(r))
            elif instruction == OpCode.PRINT:
                self.vm_print()
            elif instruction == OpCode.PRINTLN:
                self.vm_print()
                os.write(1, "\n")
            elif instruction == OpCode.POP:
                self.pop()
            elif instruction == OpCode.GLOBAL_DEF:
                idx = self.read_byte()
                self.globals.assign(idx, self.pop())
            elif instruction == OpCode.LOCAL_SET:
                idx = self.read_byte() + self.frame.frame_start
                assert idx >= 0
                # Keep the expression on the stack
                # (i.e., assignments are expressions)
                self.stack[idx] = self.peek(0)
            elif instruction == OpCode.PUSH_WIN:
                self.push(BoolValue(True))
            elif instruction == OpCode.PUSH_FAIL:
                self.push(BoolValue(False))
            elif instruction == OpCode.PUSH_NOOB:
                self.push(NOOB)
            # elif instruction == OpCode.EQ:
            #     r = self.pop()
            #     l = self.pop()
            #     self.push(l.eq(r))
            elif instruction == OpCode.RETURN:
                ret_val = self.pop()
                self.frames_top -= 1
                if self.frames_top == 0:
                    # Pop off the <script> function
                    self.pop()
                    return Result.OK

                self.stack_top = self.frame.frame_start
                idx = self.frames_top - 1
                assert idx >= 0
                self.frame = self.frames[idx]
                self.push(ret_val)
            else:
                print("Internal Error: Unknown Instruction %s" % instruction)
                return Result.RUNTIME_ERR


def init_vm_from_source(source):
    function = compile(source)

    if function is None:
        return None

    vm = Vm()
    vm.push(function)
    vm.call_value(function, 0)
    return vm


def interpret(source):
    function = compile(source)
    if function is None:
        return Result.COMPILE_ERR

    vm = Vm()
    vm.push(function)
    vm.call_value(function, 0)

    return vm.interpret()
