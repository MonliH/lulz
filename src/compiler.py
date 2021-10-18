from bytecode import Chunk, OpCode
from error import Span
from scanner import Scanner, TokenTy


class Builder:
    def __init__(self, lexer, chunk):
        self.lexer = lexer
        self.had_error = False
        self.panic_mode = False
        self.current = None
        self.previous = None

        self.chunk = chunk

    def error_at(self, token, message):
        if self.panic_mode:
            return
        self.panic_mode = True
        print("[%s] Error: %s" % (token, message))
        self.had_error = True

    def error_at_current(self, message):
        self.error_at(self.current, message)

    def advance(self):
        self.previous = self.current
        while True:
            self.current = self.lexer.scan_token()
            if self.current.ty != TokenTy.ERROR:
                break
            self.error_at_current(self.current.text)

    def consume(self, token_ty, message):
        if self.current.ty == token_ty:
            self.advance()
            return

        self.error_at_current(message)

    def compile(self):
        self.advance()
        self.consume(TokenTy.EOF, "expected end of file")
        self.end_compiler()

    def expression(self):
        pass

    def number(self):
        value = int(self.previous.text)
        self.emit_constant(value)

    def emit_constant(self, value):
        self.emit_bytes(OpCode.CONSTANT, self.make_constant(value))

    def make_constant(self, value):
        return self.chunk.add_constant(value)

    def end_compiler(self):
        self.emit_return()

    def emit_return(self):
        self.emit_byte(OpCode.RETURN)

    def emit_bytes(self, b1, b2):
        self.emit_byte(b1)
        self.emit_byte(b2)

    def emit_byte(self, b):
        self.chunk.write(b, self.previous.span)


def compile(source, chunk):
    scanner = Scanner(source)

    # while True:
    #     tok = scanner.scan_token()
    #     if not tok:
    #         # EOF
    #         break
    #     print("ty: %s, text: '%s', %s" % (tok.ty, tok.text, tok.span.str()))
    parser = Builder(scanner, chunk)
    parser.compile()
