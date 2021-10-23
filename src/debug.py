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
    os.write(
        1,
        "%s  %s:%s  "
        % (fill(str(offset), 4), fill(str(pos.s), 4), fill(str(pos.e), 4)),
    )
    if instr == OpCode.RETURN:
        return simple_instr("OP_RETURN", offset)
    elif instr == OpCode.CONSTANT:
        return const_instr("OP_CONSTANT", bytecode, offset)
    elif instr == OpCode.ADD:
        return simple_instr("OP_ADD", offset)
    elif instr == OpCode.SUB:
        return simple_instr("OP_SUB", offset)
    elif instr == OpCode.DIV:
        return simple_instr("OP_DIV", offset)
    elif instr == OpCode.MUL:
        return simple_instr("OP_MUL", offset)
    elif instr == OpCode.PRINT:
        return simple_instr("PRINT", offset)
    elif instr == OpCode.POP:
        return simple_instr("POP", offset)
    elif instr == OpCode.GLOBAL_DEF:
        return double_instr("GLOBAL_DEF", bytecode, offset)
    elif instr == OpCode.GLOBAL_GET:
        return double_instr("GLOBAL_GET", bytecode, offset)
    elif instr == OpCode.LOCAL_GET:
        return double_instr("LOCAL_GET", bytecode, offset)
    elif instr == OpCode.LOCAL_SET:
        return double_instr("LOCAL_SET", bytecode, offset)
    else:
        print("Unknown opcode %d" % instr)
        return offset + 1


def fill(s, num, c="0"):
    offset = max(0, num - len(s))
    return c * offset + s


def simple_instr(text, offset):
    print(text)
    return offset + 1


def double_instr(text, bytecode, offset):
    code_id = bytecode.code[offset + 1]
    print("%s    %s" % (text, fill(str(code_id), 4)))
    return offset + 2


def const_instr(text, bytecode, offset):
    constant = bytecode.code[offset + 1]
    print(
        "%s %s %s"
        % (
            text,
            fill(str(constant), 4),
            fill(bytecode.constants[constant].str(), 15, " "),
        )
    )
    return offset + 2
