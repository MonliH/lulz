from error import Span


class TokenTy:
    TOKEN_EOF = 0
    TOKEN_DISPLAY = 1
    TOKEN_NUMBER = 2
    TOKEN_FLOAT = 3
    TOKEN_IDENT = 4
    TOKEN_ERROR = 5
    TOKEN_STRING = 6


class Token:
    _immutable_fields_ = ["ty", "span", "text"]

    def __init__(self, ty, span, text=""):
        self.span = span
        self.ty = ty
        self.text = text


class Scanner:
    _immutable_fields_ = ["source"]

    def __init__(self, source):
        self.source = source
        self.idx = 0

    def make_token(self, ty, prev, text=""):
        return Token(ty, Span(prev, self.idx), text)

    def make_thin_token(self, ty, text=""):
        return self.make_token(ty, self.idx, text)

    def is_at_end(self):
        return self.idx >= len(self.source)

    def error_token(self, msg):
        return Token(TokenTy.TOKEN_ERROR, Span(self.idx, self.idx), msg)

    def advance(self):
        old = self.idx
        self.idx += 1
        return self.source[old]

    def peek(self):
        return self.source[self.idx]

    def match(self, expected):
        if self.is_at_end():
            return False
        if self.peek() != expected:
            return False
        self.idx += 1
        return True

    def skip_whitespace(self):
        while not self.is_at_end():
            c = self.peek()
            if c == "\n" or c == "\t" or c == " ":
                self.idx += 1
            else:
                break

    def peek_next(self):
        if self.is_at_end():
            return "\0"
        return self.source[self.idx + 1]

    def is_digit(self, c):
        return c >= "0" and c <= "9"

    def is_id_start(self, c):
        return (c >= "a" and c <= "z") or (c >= "A" and c <= "Z")

    def is_id_continue(self, c):
        return self.is_id_start(c) or c == "_" or self.is_digit(c)

    def scan_token(self):
        self.skip_whitespace()
        if self.is_at_end():
            return self.make_thin_token(TokenTy.TOKEN_EOF)

        start = self.idx
        c = self.advance()

        if c == '"':
            return self.string()
        if self.is_digit(c):
            return self.number()
        if self.is_id_start(c):
            return self.ident()

        return self.error_token("Unexpected character")

    def string(self):
        start = self.idx
        while self.peek() != '"' and not self.is_at_end():
            self.advance()

        if self.is_at_end():
            return self.error_token("Unterminated string")

        tok = self.make_token(
            TokenTy.TOKEN_STRING, start, self.source[start : self.idx]
        )

        # Eat the ending qoute
        self.advance()

        return tok

    def number(self):
        start = self.idx - 1
        assert start >= 0
        while self.is_digit(self.peek()):
            self.advance()

        if self.peek() == "." and self.is_digit(self.peek_next()):
            self.advance()
            while self.is_digit(self.peek()):
                self.advance()

            return self.make_token(
                TokenTy.TOKEN_FLOAT, start, self.source[start : self.idx]
            )

        return self.make_token(
            TokenTy.TOKEN_NUMBER, start, self.source[start : self.idx]
        )

    def ident(self):
        start = self.idx - 1
        assert start >= 0
        while self.is_id_continue(self.peek()):
            self.advance()

        return self.make_ident(start)

    def make_ident(self, start):
        slice = self.source[start : self.idx]

        first = slice[0].lower()
        if first == "d":
            if slice[1:].lower() == "isplay":
                return self.make_token(TokenTy.TOKEN_DISPLAY, start)

        return self.make_token(TokenTy.TOKEN_IDENT, start, slice)
