from bytecode import Chunk, OpCode
from error import Span
from scanner import Scanner, TokenTy
from value import IntValue


class Builder:
    def __init__(self, lexer, chunk):
        self.lexer = lexer
        self.had_error = False
        self.panic_mode = False
        self.current = None
        self.previous = None

        self.globals = {}

        self.chunk = chunk

    def error_at(self, token, message):
        if self.panic_mode:
            return
        self.panic_mode = True
        print("[%s] Error: %s" % (token.span.str(), message))
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

    def match(self, token_ty):
        if self.current.ty == token_ty:
            self.advance()
            return True
        return False

    def compile(self):
        self.advance()

        while not self.match(TokenTy.EOF):
            self.statement()

        self.end_compiler()

    def statement(self):
        if self.match(TokenTy.VISIBLE):
            self.expression()
            self.emit_byte(OpCode.PRINT)
        elif self.match(TokenTy.I):
            self.consume(TokenTy.HAS, "expected token `HAS`")
            self.consume(TokenTy.A, "expected token `A`")
            self.consume(TokenTy.IDENT, "expected identifier")
            ident = self.previous
            self.consume(TokenTy.ITZ, "expected token `ITZ`")
            self.expression()
            global_id = self.intern_global(ident.text)
            self.def_global(global_id)
        else:
            self.expression()

    def def_global(self, g_id):
        self.emit_bytes(OpCode.GLOBAL_DEF, g_id)

    def of_x_an_y(self):
        self.consume(TokenTy.OF, "expected token `OF`")
        self.expression()
        self.consume(TokenTy.AN, "expected token `AN`")
        self.expression()

    def expression(self):
        if self.match(TokenTy.SUM):
            self.of_x_an_y()
            self.emit_byte(OpCode.ADD)
        elif self.match(TokenTy.DIFF):
            self.of_x_an_y()
            self.emit_byte(OpCode.SUB)
        elif self.match(TokenTy.PRODUKT):
            self.of_x_an_y()
            self.emit_byte(OpCode.MUL)
        elif self.match(TokenTy.QUOSHUNT):
            self.of_x_an_y()
            self.emit_byte(OpCode.DIV)
        elif self.match(TokenTy.NUMBER):
            self.number()
        else:
            self.consume(TokenTy.IDENT, "expected an expression")
            self.variable()

    def variable(self):
        if self.previous.text in self.globals:
            slot = self.globals[self.previous.text]
            self.emit_bytes(OpCode.GLOBAL_GET, slot)
        else:
            self.error_at(self.previous, "undefined variable %s" % self.previous.text)

    def number(self):
        value = IntValue(int(self.previous.text))
        self.emit_constant(value)

    def intern_global(self, s):
        ident = len(self.globals)
        self.globals[s] = ident
        return ident

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
