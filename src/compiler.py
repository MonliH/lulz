from bytecode import Chunk, OpCode
from error import Span
from scanner import Scanner, Token, TokenTy
from value import IntValue


class Local:
    def __init__(self, name, depth):
        self.name = name
        self.depth = depth


class Builder:
    def __init__(self, lexer, chunk):
        self.lexer = lexer
        self.had_error = False
        self.panic_mode = False
        self.current = None
        self.previous = None

        self.globals = {}

        self.scope_depth = 0
        self.locals = []

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

        while not (self.match(TokenTy.EOF) or self.had_error):
            self.inner_block_stmt()

        self.end_compiler()

    def check(self, token_ty):
        return self.current.ty == token_ty

    def statement(self):
        if self.match(TokenTy.VISIBLE):
            self.expression()
            self.emit_byte(OpCode.PRINT)
        elif self.match(TokenTy.I):
            self.consume(TokenTy.HAS, "expected token `HAS in declaration`")
            self.consume(TokenTy.A, "expected token `A in declaration`")
            self.consume(TokenTy.IDENT, "expected identifier in declaration")
            ident = self.previous
            self.consume(TokenTy.ITZ, "expected token `ITZ in declaration`")
            self.expression()
            self.def_variable(ident.text)
        elif self.match(TokenTy.SLAB):
            self.begin_scope()
            self.block()
            self.end_scope()
        elif self.match(TokenTy.O):
            self.conditional()
        else:
            self.expression()
            # If an expression is on it's own, emit code to set the IT register
            self.write_expression()

    def conditional(self):
        self.consume(TokenTy.RLY, "expected token `RLY`")
        self.consume(TokenTy.OP_QUESTION, "expected question mark")

        else_jump = -1
        if self.match(TokenTy.YA):
            self.consume(TokenTy.RLY, "expected token `RLY`")
            jump = self.emit_jump(OpCode.JUMP_IF_FALSE)
            while not (
                self.check(TokenTy.OIC)
                or self.check(TokenTy.MEBBE)
                or self.check(TokenTy.NO)
                or self.had_error
            ):
                self.inner_block_stmt()
            else_jump = self.emit_jump(OpCode.JUMP)
            self.patch_jump(jump)

        else_if_jumps = []

        while self.match(TokenTy.MEBBE):
            self.expression()
            self.write_expression()
            jump = self.emit_jump(OpCode.JUMP_IF_FALSE)
            while not (
                self.check(TokenTy.MEBBE)
                or self.check(TokenTy.NO)
                or self.check(TokenTy.OIC)
                or self.had_error
            ):
                self.inner_block_stmt()
            else_if_jumps.append(self.emit_jump(OpCode.JUMP))
            self.patch_jump(jump)

        if self.match(TokenTy.NO):
            self.consume(TokenTy.WAI, "expected token `WAI`")
            while not (self.check(TokenTy.OIC)):
                self.inner_block_stmt()

            if else_jump != -1:
                self.patch_jump(else_jump)

        for jump in else_if_jumps:
            self.patch_jump(jump)

        self.consume(TokenTy.OIC, "expected token `OIC` to end conditional")

    def emit_jump(self, jmp_ty):
        self.emit_byte(jmp_ty)
        self.emit_byte(0)
        return len(self.chunk.code) - 1

    def patch_jump(self, offset):
        jump = len(self.chunk.code) - offset - 1
        self.chunk.code[offset] = jump

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
        elif self.match(TokenTy.WIN):
            self.emit_byte(OpCode.PUSH_WIN)
        elif self.match(TokenTy.FAIL):
            self.emit_byte(OpCode.PUSH_FAIL)
        elif self.match(TokenTy.NOOB):
            self.emit_byte(OpCode.PUSH_NOOB)
        elif self.match(TokenTy.IT):
            self.emit_byte(OpCode.GET_IT)
        else:
            self.consume(TokenTy.IDENT, "expected an expression")
            self.get_variable()

    def write_expression(self):
        self.emit_byte(OpCode.SET_IT)

    def inner_block_stmt(self):
        self.statement()
        # Optional comma at the end of a line
        self.match(TokenTy.OP_COMMA)

    def block(self):
        while not (
            self.check(TokenTy.KILL) or self.check(TokenTy.EOF) or self.had_error
        ):
            self.inner_block_stmt()
        self.consume(TokenTy.KILL, "expected token `KILL` after block")

    def begin_scope(self):
        self.scope_depth += 1

    def end_scope(self):
        self.scope_depth -= 1
        while len(self.locals) and self.locals[-1].depth > self.scope_depth:
            self.emit_byte(OpCode.POP)
            self.locals.pop()

    def def_variable(self, ident):
        if self.scope_depth > 0:
            self.dec_variable(ident)
            return
        g_id = self.intern_global(ident)
        self.emit_bytes(OpCode.GLOBAL_DEF, g_id)

    def of_x_an_y(self):
        self.consume(TokenTy.OF, "expected token `OF` in operator")
        self.expression()
        self.consume(TokenTy.AN, "expected token `AN` in operator")
        self.expression()

    def resolve_local(self, name):
        for i in range(len(self.locals) - 1, -1, -1):
            local = self.locals[i]
            if local.name == name:
                return i
        return -1

    def get_variable(self):
        var_pos = self.resolve_local(self.previous.text)
        if var_pos != -1:
            # Local exists
            self.emit_bytes(OpCode.LOCAL_GET, var_pos)
            return

        if self.previous.text in self.globals:
            # Global exists
            slot = self.globals[self.previous.text]
            self.emit_bytes(OpCode.GLOBAL_GET, slot)
            return

        self.error_at(self.previous, "undefined variable %s" % self.previous.text)

    def dec_variable(self, name):
        for i in range(len(self.locals) - 1, -1, -1):
            local = self.locals[i]
            if local.depth != -1 and local.depth > self.scope_depth:
                break

            if local.name == name:
                self.error_at(
                    self.previous,
                    "variable with this name already declared in this scope",
                )

        self.add_local(name)

    def add_local(self, name):
        self.locals.append(Local(name, self.scope_depth))

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
    if parser.had_error:
        return None
    return chunk
