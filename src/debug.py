from bytecode import OpCode
import os


def disassemble(bytecode, name):
    # f-strings don't work with rpython :(
    print("== %s ==" % name)
    offset = 0
    while offset < len(bytecode.code):
        offset = disassemble_instr(bytecode, offset)


def disassemble_instr(bytecode, offset):
    instr = bytecode.code[offset]
    pos = bytecode.pos[offset]
    os.write(1, "%s  %s:%s  " % (fill(str(offset), 4), fill(str(pos.s), 4), fill(str(pos.e), 4)))
    if instr == OpCode.OP_RETURN:
        return simple_instr("OP_RETURN", offset)
    elif instr == OpCode.OP_CONSTANT:
        return const_instr("OP_CONSTANT", bytecode, offset)
    else:
        print("Unknown opcode %d" % instr)
        return offset + 1


def fill(s, num, c="0"):
    offset = max(0, num - len(s))
    return c * offset + s


def simple_instr(text, offset):
    print(text)
    return offset + 1


def const_instr(text, bytecode, offset):
    constant = bytecode.code[offset + 1]
    print("%s %s %s" % (text, fill(str(constant), 4), fill(bytecode.constants[constant].str(), 15, " ")))
    return offset + 2
