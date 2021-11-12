from error import Span


def enum(*sequential, **named):
    enums = dict(zip(sequential, range(len(sequential))), **named)
    reverse = dict((value, key) for key, value in enums.items())
    return (type("Enum", (), enums), reverse)


(TokenTy, token_ty_map) = enum(
    "EOF",
    "NUMBER",
    "FLOAT",
    "IDENT",
    "ERROR",
    "STRING",
    "OP_QUESTION",
    "OP_BANG",
    "BREAK",
    "A",
    "ALL",
    "AN",
    "ANY",
    "BIGGR",
    "BOTH",
    "DIFF",
    "DIFFRINT",
    "EITHER",
    "EQ",
    "FAIL",
    "FOUND",
    "GIMMEH",
    "GRETER",
    "GTFO",
    "HAI",
    "HAS",
    "HOW",
    "I",
    "IF",
    "IM",
    "IN",
    "IS",
    "IT",
    "ITZ",
    "IZ",
    "KILL",
    "KTHXBYE",
    "LES",
    "MAEK",
    "MEBBE",
    "MKAY",
    "MOD",
    "NO",
    "NOOB",
    "NOT",
    "NOW",
    "O",
    "OF",
    "OIC",
    "OMG",
    "OMGWTF",
    "OUTTA",
    "PRODUKT",
    "QUOSHUNT",
    "R",
    "RLY",
    "SAEM",
    "SAY",
    "SLAB",
    "SMALLR",
    "SMOOSH",
    "SO",
    "SUM",
    "THEN",
    "TIL",
    "U",
    "UR",
    "VISIBLE",
    "WAI",
    "WILE",
    "WIN",
    "WON",
    "WTF",
    "YA",
    "YR",
)


token_map = {
    "a": TokenTy.A,
    "all": TokenTy.ALL,
    "an": TokenTy.AN,
    "any": TokenTy.ANY,
    "biggr": TokenTy.BIGGR,
    "both": TokenTy.BOTH,
    "diff": TokenTy.DIFF,
    "diffrint": TokenTy.DIFFRINT,
    "either": TokenTy.EITHER,
    "found": TokenTy.FOUND,
    "gimmeh": TokenTy.GIMMEH,
    "gtfo": TokenTy.GTFO,
    "has": TokenTy.HAS,
    "how": TokenTy.HOW,
    "i": TokenTy.I,
    "if": TokenTy.IF,
    "im": TokenTy.IM,
    "in": TokenTy.IN,
    "is": TokenTy.IS,
    "iz": TokenTy.IZ,
    "it": TokenTy.IT,
    "itz": TokenTy.ITZ,
    "maek": TokenTy.MAEK,
    "mebbe": TokenTy.MEBBE,
    "mkay": TokenTy.MKAY,
    "mod": TokenTy.MOD,
    "no": TokenTy.NO,
    "not": TokenTy.NOT,
    "now": TokenTy.NOW,
    "o": TokenTy.O,
    "of": TokenTy.OF,
    "omg": TokenTy.OMG,
    "omgwtf": TokenTy.OMGWTF,
    "outta": TokenTy.OUTTA,
    "produkt": TokenTy.PRODUKT,
    "quoshunt": TokenTy.QUOSHUNT,
    "r": TokenTy.R,
    "rly": TokenTy.RLY,
    "saem": TokenTy.SAEM,
    "say": TokenTy.SAY,
    "smallr": TokenTy.SMALLR,
    "smoosh": TokenTy.SMOOSH,
    "so": TokenTy.SO,
    "sum": TokenTy.SUM,
    "til": TokenTy.TIL,
    "u": TokenTy.U,
    "ur": TokenTy.UR,
    "visible": TokenTy.VISIBLE,
    "wai": TokenTy.WAI,
    "wile": TokenTy.WILE,
    "won": TokenTy.WON,
    "wtf": TokenTy.WTF,
    "ya": TokenTy.YA,
    "yr": TokenTy.YR,
    "kill": TokenTy.KILL,
    "slab": TokenTy.SLAB,
    "win": TokenTy.WIN,
    "fail": TokenTy.FAIL,
    "noob": TokenTy.NOOB,
    "oic": TokenTy.OIC,
    "hai": TokenTy.HAI,
    "kthxbye": TokenTy.KTHXBYE,
    "les": TokenTy.LES,
    "greter": TokenTy.GRETER,
    "eq": TokenTy.EQ,
    "then": TokenTy.THEN,
}


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

    def skip_newlines(self):
        while not self.is_at_end():
            c = self.peek()
            if c == "\n":
                self.idx += 1
            else:
                break

    def skip_whitespace(self):
        while not self.is_at_end():
            c = self.peek()
            if c == "\t" or c == " ":
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
        if c == "?":
            return self.make_token(TokenTy.OP_QUESTION, start)
        if c == "!":
            return self.make_token(TokenTy.OP_BANG, start)
        if c == ",":
            return self.make_token(TokenTy.BREAK, start)
        if self.is_digit(c):
            return self.number()
        if self.is_id_start(c):
            return self.ident()
        if c == "\n":
            self.skip_newlines()
            return self.make_token(TokenTy.BREAK, start)

        return self.error_token("unexpected character `%s`" % c)

    def string(self):
        start = self.idx
        while self.peek() != '"' and not self.is_at_end():
            self.advance()

        if self.is_at_end():
            return self.error_token("unterminated string")

        tok = self.make_token(TokenTy.STRING, start, self.source[start : self.idx])

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

            return self.make_token(TokenTy.FLOAT, start, self.source[start : self.idx])

        return self.make_token(TokenTy.NUMBER, start, self.source[start : self.idx])

    def ident(self):
        start = self.idx - 1
        assert start >= 0
        while self.is_id_continue(self.peek()):
            self.advance()

        return self.make_ident(start)

    def make_ident(self, start):
        slice = self.source[start : self.idx]

        lower = slice.lower()
        if lower == "btw":
            # Single line comment
            while self.peek() != "\n" and not self.is_at_end():
                self.advance()
            return self.scan_token()
        if lower in token_map:
            return self.make_token(token_map[lower], start)

        return self.make_token(TokenTy.IDENT, start, slice)
