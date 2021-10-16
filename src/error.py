class Span:
    _immutable_fields_ = ["s", "e"]

    def __init__(self, s, e):
        self.s = s
        self.e = e

    def str(self):
        return "%s:%s" % (self.s, self.e)

    def __str__(self):
        return self.str()

    def __repr__(self):
        return self.str()
