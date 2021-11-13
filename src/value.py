from rpython.rlib.objectmodel import instantiate


class Value(object):
    __slots__ = ()

    def get_value(self):
        return self

    def __deepcopy__(self):
        return instantiate(self.__class__)

    def type(self):
        return ""

    def dbg(self):
        return "%s of type %s" % (self.str(), self.type())

    def str(self):
        return ""

class BoolValue(Value):
    __slots__ = ("bool_val",)
    _immutable_fields_ = ["bool_val"]

    def __init__(self, bool_val):
        assert isinstance(bool_val, bool)
        self.bool_val = bool_val

    def type(self):
        return "TROOF"

    def str(self):
        return "WIN" if self.bool_val else "FAIL"


class NullValue(Value):
    __slots__ = ()
    _immutable_fields_ = []

    def __init__(self):
        pass

    def str(self):
        return "NOOB"

    def type(self):
        return "NOOB"


class IntValue(Value):
    __slots__ = ("int_val",)
    _immutable_fields_ = ["int_val"]

    def __init__(self, int_val):
        assert isinstance(int_val, int)
        self.int_val = int_val

    def str(self):
        return str(self.int_val)

    def type(self):
        return "NUMBR"


class FloatValue(Value):
    __slots__ = ("float_val",)
    _immutable_fields_ = ["float_val"]

    def __init__(self, float_val):
        assert isinstance(float_val, float)
        self.float_val = float_val

    def str(self):
        return str(self.float_val).rstrip("0").rstrip(".")

    def type(self):
        return "NUMBAR"


class StrValue(Value):
    __slots__ = ("str_val",)
    _immutable_fields_ = ["str_val"]

    def __init__(self, str_val):
        assert isinstance(str_val, str)
        self.str_val = str_val

    def str(self):
        return self.str_val

    def type(self):
        return "YARN"


class FuncValue(Value):
    __slots__ = ("arity", "chunk", "name")
    _immutable_fields_ = ["arity", "name"]

    def __init__(self, arity, chunk, name):
        assert isinstance(arity, int)
        assert isinstance(name, str)
        self.arity = arity
        self.chunk = chunk
        self.name = name

    def type(self):
        return "FUNKSHUN"

    def str(self):
        return "<FUNKSHUN %s>" % self.name
