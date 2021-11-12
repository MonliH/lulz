from bytecode import OpCode
import os


def disassemble(bytecode, name):
    # f-strings don't work with rpython :(
    os.write(2, "== %s ==\n" % name)
    offset = 0
    while offset < len(bytecode.code):
        offset = disassemble_instr(bytecode, offset)


def disassemble_instr(bytecode, offset):
    instr = bytecode.code[offset]
    pos = bytecode.pos[offset]
    os.write(
        2,
        "%s  %s:%s  "
        % (zfill(str(offset), 4), zfill(str(pos.s), 4), zfill(str(pos.e), 4)),
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
        return double_instr("PRINT", bytecode, offset)
    elif instr == OpCode.PRINTLN:
        return double_instr("PRINTLN", bytecode, offset)
    elif instr == OpCode.POP:
        return simple_instr("POP", offset)
    elif instr == OpCode.PUSH_WIN:
        return simple_instr("PUSH_WIN", offset)
    elif instr == OpCode.PUSH_FAIL:
        return simple_instr("PUSH_FAIL", offset)
    elif instr == OpCode.PUSH_NOOB:
        return simple_instr("PUSH_NOOB", offset)
    elif instr == OpCode.GLOBAL_DEF:
        return double_instr("GLOBAL_DEF", bytecode, offset)
    elif instr == OpCode.GLOBAL_GET:
        return double_instr("GLOBAL_GET", bytecode, offset)
    elif instr == OpCode.LOCAL_GET:
        return double_instr("LOCAL_GET", bytecode, offset)
    elif instr == OpCode.LOCAL_SET:
        return double_instr("LOCAL_SET", bytecode, offset)
    elif instr == OpCode.JUMP_IF_FALSE:
        return double_instr("JUMP_IF_FALSE", bytecode, offset)
    elif instr == OpCode.JUMP:
        return double_instr("JUMP", bytecode, offset)
    elif instr == OpCode.SET_IT:
        return simple_instr("SET_IT", offset)
    elif instr == OpCode.GET_IT:
        return simple_instr("GET_IT", offset)
    elif instr == OpCode.MIN:
        return simple_instr("MIN", offset)
    elif instr == OpCode.MAX:
        return simple_instr("MAX", offset)
    elif instr == OpCode.EQ:
        return simple_instr("EQ", offset)
    elif instr == OpCode.CALL:
        return double_instr("CALL", bytecode, offset)
    else:
        os.write(2, "Unknown opcode %d\n" % instr)
        return offset + 1


def fill(s, num, c):
    offset = max(0, num - len(s))
    return c * offset + s

def zfill(s, num):
    return fill(s, num, "0")


def simple_instr(text, offset):
    os.write(2, "%s\n" % text)
    return offset + 1


def double_instr(text, bytecode, offset):
    code_id = bytecode.code[offset + 1]
    os.write(2, "%s    %s\n" % (text, zfill(str(code_id), 4)))
    return offset + 2


def const_instr(text, bytecode, offset):
    constant = bytecode.code[offset + 1]
    os.write(
        2,
        "%s %s %s\n"
        % (
            text,
            zfill(str(constant), 4),
            fill(bytecode.constants[constant].str(), 15, " "),
        ),
    )
    return offset + 2
