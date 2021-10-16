from bytecode import Chunk, OpCode
from error import Span
from scanner import Scanner, TokenTy


def compile(source):
    scanner = Scanner(source)
    while True:
        tok = scanner.scan_token()
        print("ty: %s, text: '%s', %s" % (tok.ty, tok.text, tok.span.str()))
        if tok.ty == TokenTy.EOF:
            break
    return Chunk([], [OpCode.OP_RETURN], [Span(0, 0)])
