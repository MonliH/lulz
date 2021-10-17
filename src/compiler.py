from bytecode import Chunk, OpCode
from error import Span
from scanner import Scanner, TokenTy


def compile(source):
    scanner = Scanner(source)
    while True:
        tok = scanner.scan_token()
        if not tok:
            # EOF
            break
        print("ty: %s, text: '%s', %s" % (tok.ty, tok.text, tok.span.str()))
    return Chunk([], [OpCode.OP_RETURN], [Span(0, 0)])
