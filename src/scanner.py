from error import Span


# Oh my goodness, lolcode has so many reserved keywords
class TokenTy:
    EOF = 0
    NUMBER = 2
    FLOAT = 3
    IDENT = 4
    ERROR = 5
    STRING = 6

    OP_QUESTION = 7
    OP_BANG = 8
    OP_COMMA = 9

    A = 10
    ALL = 11
    AN = 12
    ANY = 13
    BIGGR = 14
    BOTH = 15
    DIFF = 17
    DIFFRINT = 18
    EITHER = 19
    FOUND = 20
    GIMMEH = 21
    GTFO = 22
    HOW = 23
    I = 24
    IF = 25
    IM = 26
    IN = 27
    IS = 28
    IZ = 29
    MAEK = 30
    MEBBE = 31
    MKAY = 32
    MOD = 33
    NO = 34
    NOT = 35
    NOW = 36
    O = 37
    OF = 38
    OMG = 39
    OMGWTF = 40
    OUTTA = 41
    PRODUKT = 42
    QUOSHUNT = 43
    R = 44
    RLY = 45
    SAEM = 46
    SAY = 47
    SMALLR = 48
    SMOOSH = 49
    SO = 50
    SUM = 51
    TIL = 52
    U = 53
    UR = 54
    VISIBLE = 55
    WAI = 56
    WILE = 57
    WON = 58
    WTF = 59
    YA = 60
    YR = 61


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
        return Token(TokenTy.ERROR, Span(self.idx, self.idx), msg)

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
            return self.make_thin_token(TokenTy.EOF)

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
            TokenTy.STRING, start, self.source[start : self.idx]
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
                TokenTy.FLOAT, start, self.source[start : self.idx]
            )

        return self.make_token(
            TokenTy.NUMBER, start, self.source[start : self.idx]
        )

    def ident(self):
        start = self.idx - 1
        assert start >= 0
        while self.is_id_continue(self.peek()):
            self.advance()

        return self.make_ident(start)

    def make_ident(self, start):
        slice = self.source[start : self.idx]

        lower = slice.lower()
        if lower[0] == "a":
            if len(lower) == 1:
                return self.make_token(TokenTy.A, start)
            if lower[1] == "n":
                if len(lower) == 2:
                    return self.make_token(TokenTy.AN, start)
                if lower[2:] == "y":
                    return self.make_token(TokenTy.ANY, start)
            if lower[1] == "l":
                if lower[2:] == "l":
                    return self.make_token(TokenTy.ALL, start)
        if lower[0] == "b":
            if lower[1] == "i" and lower[2:] == "ggr":
                return self.make_token(TokenTy.BIGGR, start)
            if lower[1] == "o" and lower[2:] == "th":
                return self.make_token(TokenTy.BOTH, start)
        if lower[0] == "d":
            if lower[1:4] == "iff":
                if len(lower) == 4:
                    return self.make_token(TokenTy.DIFF, start)
                if lower[4:] == "rint":
                    return self.make_token(TokenTy.DIFFRINT, start)
        if lower[0] == "v" and lower[1:] == "isible":
                return self.make_token(TokenTy.VISIBLE, start)

        return self.make_token(TokenTy.IDENT, start, slice)
