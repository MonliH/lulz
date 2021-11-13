from bytecode import Chunk, OpCode
from debug import disassemble
from error import Span
from scanner import Scanner, Token, TokenTy, token_ty_map
from value import FloatValue, FuncValue, IntValue, StrValue
import os


class Local:
    def __init__(self, name, depth):
        self.name = name
        self.depth = depth


class FunctionTy:
    FUNCTION = 0
    SCRIPT = 1


class Builder:
    def __init__(self, lexer, ty, prev, name="<script>"):
        self.lexer = lexer
        self.had_error = False
        self.panic_mode = False
        self.current = None
        self.previous = None
        self.enclosing = prev
        self.ty = ty

        self.globals = {}

        self.scope_depth = 0
        self.locals = []

        self.add_local("")

        self.fn = FuncValue(0, Chunk([], [], []), name)

    def error_at(self, token, message):
        if self.panic_mode:
            return
        self.panic_mode = True
        os.write(2, "[%s] Error: %s, IDIOT!\n" % (token.span.str(), message))
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

        self.error_at_current("%s, found %s" % (message, token_ty_map[self.current.ty]))

    def match(self, token_ty):
        if self.current.ty == token_ty:
            self.advance()
            return True
        return False

    def check_prev(self, token_ty):
        return self.previous.ty == token_ty

    def compile(self):
        self.advance()
        self.eat_break()
        self.consume(TokenTy.HAI, "expected `HAI` at start of code")
        self.consume(TokenTy.FLOAT, "expected version number after `HAI`")
        self.eat_break()

        while not (self.is_at_end() or self.had_error):
            self.inner_block_stmt()

        self.end_compiler()
        if not self.had_error:
            self.eat_break()
            self.consume(TokenTy.KTHXBYE, "expected `KTHXBYE` at end of code")
            self.eat_break()

        return self.fn

    def check(self, token_ty):
        return self.current.ty == token_ty

    def statement(self):
        self.eat_break()

        if self.match(TokenTy.HOW):
            self.func_declaration()
        elif self.match(TokenTy.FOUND):
            if self.ty == FunctionTy.SCRIPT:
                self.error_at_current("cannot return from top-level code")
            self.consume(TokenTy.YR, "expected token `YR`")
            self.expression()
            self.emit_return()
        elif self.match(TokenTy.GTFO):
            if self.ty == FunctionTy.SCRIPT:
                self.error_at_current("cannot return from top-level code")
            self.emit_byte(OpCode.PUSH_NOOB)
            self.emit_return()
        elif self.match(TokenTy.VISIBLE):
            amount = 1
            self.expression()
            while not (self.is_at_end() or self.had_error or self.check(TokenTy.BREAK) or self.check(TokenTy.OP_BANG)):
                self.expression()
                amount += 1
            if self.match(TokenTy.OP_BANG):
                self.emit_byte(OpCode.PRINT)
            else:
                self.emit_byte(OpCode.PRINTLN)
                self.line_break()
            self.emit_byte(amount)
        elif self.match(TokenTy.I) and self.check(TokenTy.HAS):
            self.consume(TokenTy.HAS, "expected token `HAS` in declaration")
            self.consume(TokenTy.A, "expected token `A` in declaration")
            ident = self.ident()
            self.consume(TokenTy.ITZ, "expected token `ITZ` in declaration")
            self.expression()
            self.def_variable(ident.text)
            self.line_break()
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
            self.line_break()

    def func_declaration(self):
        # HOW IZ I <name>
        # <body>
        # IF U SAY SO
        self.consume(TokenTy.IZ, "expected token `IZ`")
        self.consume(TokenTy.I, "expected token `I`")
        fn_name = self.ident()
        self.function(FunctionTy.FUNCTION, fn_name.text)
        self.consume(TokenTy.IF, "expected token `IF`")
        self.consume(TokenTy.U, "expected token `U`")
        self.consume(TokenTy.SAY, "expected token `SAY`")
        self.consume(TokenTy.SO, "expected token `SO`")

    def function_args(self):
        arity = 0
        if self.match(TokenTy.YR):
            arity = 1
            ident = self.ident().text
            self.add_local(ident)
            self.eat_break()
            while self.match(TokenTy.AN):
                arity += 1
                self.consume(TokenTy.YR, "expected token `YR`")
                self.add_local(self.ident().text)
                self.eat_break()
        return arity

    def function(self, ty, name):
        compiler = Builder(self.lexer, ty, self, name)
        compiler.globals = self.globals
        compiler.previous = self.previous
        compiler.current = self.current
        compiler.scope_depth = self.scope_depth

        compiler.begin_scope()
        compiler.eat_break()
        arity = compiler.function_args()
        global_id = compiler.intern_global(name)

        while not (compiler.is_at_end() or compiler.had_error or compiler.check(TokenTy.IF)):
            compiler.inner_block_stmt()

        function = compiler.end_compiler()
        function.arity = arity
        self.previous = compiler.previous
        self.current = compiler.current
        self.emit_constant(function)
        self.emit_bytes(OpCode.GLOBAL_DEF, global_id)
        if compiler.had_error:
            self.had_error = True

    def line_break(self):
        self.consume(TokenTy.BREAK, "expected line break")

    def eat_break(self):
        while self.match(TokenTy.BREAK):
            pass

    def conditional(self):
        self.consume(TokenTy.RLY, "expected token `RLY`")
        self.consume(TokenTy.OP_QUESTION, "expected question mark")

        self.eat_break()

        else_jump = -1
        if self.match(TokenTy.YA):
            self.consume(TokenTy.RLY, "expected token `RLY`")
            self.eat_break()
            jump = self.emit_jump(OpCode.JUMP_IF_FALSE)
            while not (
                self.check(TokenTy.OIC)
                or self.check(TokenTy.MEBBE)
                or self.check(TokenTy.NO)
                or self.is_at_end()
                or self.had_error
            ):
                self.inner_block_stmt()
            else_jump = self.emit_jump(OpCode.JUMP)
            self.patch_jump(jump)

        self.eat_break()

        else_if_jumps = []

        while self.match(TokenTy.MEBBE):
            self.eat_break()
            self.expression()
            self.eat_break()
            self.write_expression()
            jump = self.emit_jump(OpCode.JUMP_IF_FALSE)
            while not (
                self.check(TokenTy.MEBBE)
                or self.check(TokenTy.NO)
                or self.check(TokenTy.OIC)
                or self.had_error
                or self.is_at_end()
            ):
                self.inner_block_stmt()
            else_if_jumps.append(self.emit_jump(OpCode.JUMP))
            self.patch_jump(jump)

        self.eat_break()

        if self.match(TokenTy.NO):
            self.consume(TokenTy.WAI, "expected token `WAI`")
            self.eat_break()
            while not (self.check(TokenTy.OIC) or self.is_at_end()):
                self.inner_block_stmt()

            if else_jump != -1:
                self.patch_jump(else_jump)

        for jump in else_if_jumps:
            self.patch_jump(jump)

        self.eat_break()

        self.consume(TokenTy.OIC, "expected token `OIC` to end conditional")

    def current_chunk(self):
        return self.fn.chunk

    def emit_jump(self, jmp_ty):
        self.emit_byte(jmp_ty)
        self.emit_byte(0)
        return len(self.current_chunk().code) - 1

    def patch_jump(self, offset):
        jump = len(self.current_chunk().code) - offset - 1
        self.current_chunk().code[offset] = jump

    def expression(self):
        if self.check_prev(TokenTy.I) or self.match(TokenTy.I):
            self.call()
        elif self.match(TokenTy.STRING):
            self.emit_constant(StrValue(self.previous.text))
        elif self.match(TokenTy.SUM):
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
        elif self.match(TokenTy.BIGGR):
            self.of_x_an_y()
            self.emit_byte(OpCode.MAX)
        elif self.match(TokenTy.SMALLR):
            self.of_x_an_y()
            self.emit_byte(OpCode.MIN)
        elif self.match(TokenTy.NUMBER):
            self.number()
        elif self.match(TokenTy.FLOAT):
            self.float()
        elif self.match(TokenTy.WIN):
            self.emit_byte(OpCode.PUSH_WIN)
        elif self.match(TokenTy.FAIL):
            self.emit_byte(OpCode.PUSH_FAIL)
        elif self.match(TokenTy.NOOB):
            self.emit_byte(OpCode.PUSH_NOOB)
        elif self.match(TokenTy.IT):
            self.emit_byte(OpCode.GET_IT)
        elif self.match(TokenTy.BOTH):
            self.consume(TokenTy.SAEM, "expected token `SAEM` after `BOTH`")
            self.expression()
            self.match(TokenTy.AN)
            self.expression()
            self.emit_byte(OpCode.EQ)
        elif self.match(TokenTy.IZ):
            self.cmp_op()
        else:
            self.consume(TokenTy.IDENT, "expected an expression")
            varname = self.previous.text
            if self.match(TokenTy.R):
                # Variable Assignment
                self.expression()
                self.set_variable(varname)
            else:
                self.get_variable(varname)

    def call(self):
        self.consume(TokenTy.IZ, "expected token `IZ`")
        fn = self.expression()
        args = self.argument_list()
        self.emit_bytes(OpCode.CALL, args)

    def argument_list(self):
        arg_count = 0
        if self.match(TokenTy.YR):
            self.expression()
            arg_count = 1
            while self.match(TokenTy.AN):
                self.consume(TokenTy.YR, "expected token `YR`")
                self.expression()
                arg_count += 1

        self.consume(TokenTy.MKAY, "expected token `MKAY` after arguments")

        return arg_count

    def ident(self):
        self.consume(TokenTy.IDENT, "expected identifier")
        return self.previous

    def cmp_op(self):
        self.expression()
        if self.match(TokenTy.LES):
            byte = OpCode.LTE if self.match(TokenTy.EQ) else OpCode.LT
            self.consume(TokenTy.THEN, "expected token `THEN`")
            self.expression()
            self.emit_byte(byte)
        elif self.match(TokenTy.GRETER):
            byte = OpCode.GTE if self.match(TokenTy.EQ) else OpCode.GT
            self.consume(TokenTy.THEN, "expected token `THEN`")
            self.expression()
            self.emit_byte(byte)
        else:
            self.error_at_current("expected comparison operator")

    def write_expression(self):
        self.emit_byte(OpCode.SET_IT)

    def inner_block_stmt(self):
        self.statement()
        # Optional break at end of a line
        self.eat_break()

    def block(self):
        while not (self.check(TokenTy.KILL) or self.is_at_end() or self.had_error):
            self.inner_block_stmt()
        self.consume(TokenTy.KILL, "expected token `KILL` after block")

    def is_at_end(self):
        return self.check(TokenTy.EOF) or self.check(TokenTy.KTHXBYE)

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

    def set_variable(self, varname):
        var_pos = self.resolve_local(varname)
        if var_pos != -1:
            # Local exists
            self.emit_bytes(OpCode.LOCAL_SET, var_pos)
            return

        if varname in self.globals:
            # Global exists
            slot = self.globals[varname]
            self.emit_bytes(OpCode.GLOBAL_DEF, slot)
            return

        self.error_at(self.previous, "undefined variable %s" % varname)

    def get_variable(self, varname):
        var_pos = self.resolve_local(varname)
        if var_pos != -1:
            # Local exists
            self.emit_bytes(OpCode.LOCAL_GET, var_pos)
            return

        if varname in self.globals:
            # Global exists
            slot = self.globals[varname]
            self.emit_bytes(OpCode.GLOBAL_GET, slot)
            return

        self.error_at(self.previous, "undefined variable %s" % varname)

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

    def float(self):
        value = FloatValue(float(self.previous.text))
        self.emit_constant(value)

    def intern_global(self, s):
        ident = len(self.globals)
        self.globals[s] = ident
        return ident

    def emit_constant(self, value):
        self.emit_bytes(OpCode.CONSTANT, self.make_constant(value))

    def make_constant(self, value):
        return self.current_chunk().add_constant(value)

    def end_compiler(self):
        self.emit_byte(OpCode.GET_IT)
        self.emit_return()
        disassemble(self.current_chunk(), self.fn.name)
        return self.fn

    def emit_return(self):
        self.emit_byte(OpCode.RETURN)

    def emit_bytes(self, b1, b2):
        self.emit_byte(b1)
        self.emit_byte(b2)

    def emit_byte(self, b):
        self.current_chunk().write(b, self.previous.span)


def compile(source):
    scanner = Scanner(source)

    # while True:
    #     tok = scanner.scan_token()
    #     if not tok:
    #         # EOF
    #         break
    #     print("ty: %s, text: '%s', %s" % (tok.ty, tok.text, tok.span.str()))
    parser = Builder(scanner, FunctionTy.SCRIPT, None)
    function = parser.compile()
    if parser.had_error:
        return None
    return function
