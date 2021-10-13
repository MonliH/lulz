from bytecode import OpCode
import os


def disassemble(bytecode, name):
    # f-strings don't work with rpython :(
    print("== %s ==" % name)
    offset = 0
    while offset < len(bytecode.ops):
        offset = disassemble_instr(bytecode, offset)


def disassemble_instr(bytecode, offset):
    os.write(1, "%s " % zfill(str(offset), 4))

    instr = bytecode.ops[offset]
    if instr == OpCode.OP_RETURN:
        return simple_instr("OP_RETURN", offset)
    else:
        print("Unknown opcode %d" % instr)
        return offset + 1


def zfill(s, num):
    offset = max(0, num - len(s))
    return "0" * offset + s


def simple_instr(text, offset):
    print(text)
    return offset + 1
