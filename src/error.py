class Span:
    _immutable_fields_ = ["s", "e"]

    def __init__(self, s, e):
        self.s = s
        self.e = e

