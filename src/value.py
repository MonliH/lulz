from rpython.rlib.objectmodel import instantiate


class Value:
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

    def add(self, other):
        return self

    def sub(self, other):
        return self

    def mul(self, other):
        return self

    def div(self, other):
        return self

    def eq(self, other):
        return BoolValue(False)

    def is_truthy(self):
        return False

    def gt(self, other):
        return BoolValue(False)

    def lt(self, other):
        return BoolValue(False)

    def gte(self, other):
        return BoolValue(False)

    def lte(self, other):
        return BoolValue(False)

    def to_number(self):
        return None

    def min(self, other):
        return None

    def max(self, other):
        return None


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

    def eq(self, other):
        if isinstance(other, BoolValue):
            return BoolValue(self.bool_val == other.bool_val)
        return BoolValue(False)

    def is_truthy(self):
        return self.bool_val

    def to_number(self):
        return IntValue(1 if self.bool_val else 0)


class NullValue(Value):
    __slots__ = ()
    _immutable_fields_ = []

    def __init__(self):
        pass

    def str(self):
        return "NOOB"

    def type(self):
        return "NOOB"

    def eq(self, other):
        if isinstance(other, NullValue):
            return BoolValue(True)
        return BoolValue(False)

    def is_truthy(self):
        return False

    def to_number(self):
        return IntValue(0)


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

    def add(self, other):
        if isinstance(other, IntValue):
            return IntValue(self.int_val + other.int_val)
        elif isinstance(other, FloatValue):
            return FloatValue(self.int_val + other.float_val)
        assert False

    def mul(self, other):
        if isinstance(other, IntValue):
            return IntValue(self.int_val * other.int_val)
        elif isinstance(other, FloatValue):
            return FloatValue(self.int_val * other.float_val)
        assert False

    def div(self, other):
        if isinstance(other, IntValue):
            return IntValue(self.int_val // other.int_val)
        elif isinstance(other, FloatValue):
            return FloatValue(self.int_val / other.float_val)
        assert False

    def sub(self, other):
        if isinstance(other, IntValue):
            return IntValue(self.int_val - other.int_val)
        elif isinstance(other, FloatValue):
            return FloatValue(self.int_val - other.float_val)
        assert False

    def lt(self, other):
        if isinstance(other, IntValue):
            return IntValue(self.int_val < other.int_val)
        elif isinstance(other, FloatValue):
            return FloatValue(self.int_val < other.float_val)
        assert False

    def lte(self, other):
        if isinstance(other, IntValue):
            return IntValue(self.int_val <= other.int_val)
        elif isinstance(other, FloatValue):
            return FloatValue(self.int_val <= other.float_val)
        assert False

    def gte(self, other):
        if isinstance(other, IntValue):
            return IntValue(self.int_val >= other.int_val)
        elif isinstance(other, FloatValue):
            return FloatValue(self.int_val >= other.float_val)
        assert False

    def gt(self, other):
        if isinstance(other, IntValue):
            return IntValue(self.int_val > other.int_val)
        elif isinstance(other, FloatValue):
            return FloatValue(self.int_val > other.float_val)
        assert False

    def eq(self, other):
        if isinstance(other, IntValue):
            return BoolValue(self.int_val == other.int_val)
        elif isinstance(other, FloatValue):
            return BoolValue(float(self.int_val) == other.float_val)
        return BoolValue(False)

    def min(self, other):
        if isinstance(other, IntValue):
            return IntValue(min(self.int_val, other.int_val))
        elif isinstance(other, FloatValue):
            return FloatValue(min(float(self.int_val), other.float_val))
        assert False

    def max(self, other):
        if isinstance(other, IntValue):
            return IntValue(max(self.int_val, other.int_val))
        elif isinstance(other, FloatValue):
            return FloatValue(max(float(self.int_val), other.float_val))
        assert False

    def is_truthy(self):
        return self.int_val != 0

    def to_number(self):
        return self


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

    def add(self, other):
        if isinstance(other, IntValue):
            return FloatValue(self.float_val + other.int_val)
        elif isinstance(other, FloatValue):
            return FloatValue(self.float_val + other.float_val)
        assert False

    def div(self, other):
        if isinstance(other, IntValue):
            return FloatValue(self.float_val / other.int_val)
        elif isinstance(other, FloatValue):
            return FloatValue(self.float_val / other.float_val)
        assert False

    def mul(self, other):
        if isinstance(other, IntValue):
            return FloatValue(self.float_val * other.int_val)
        elif isinstance(other, FloatValue):
            return FloatValue(self.float_val * other.float_val)
        assert False

    def sub(self, other):
        if isinstance(other, IntValue):
            return FloatValue(self.float_val - other.int_val)
        elif isinstance(other, FloatValue):
            return FloatValue(self.float_val - other.float_val)
        assert False

    def eq(self, other):
        if isinstance(other, FloatValue):
            return BoolValue(self.float_val == other.float_val)
        elif isinstance(other, IntValue):
            return BoolValue(self.float_val == float(other.int_val))
        return BoolValue(False)

    def lt(self, other):
        if isinstance(other, IntValue):
            return BoolValue(self.float_val < other.int_val)
        elif isinstance(other, FloatValue):
            return BoolValue(self.float_val < other.float_val)
        assert False

    def lte(self, other):
        if isinstance(other, IntValue):
            return BoolValue(self.float_val <= other.int_val)
        elif isinstance(other, FloatValue):
            return BoolValue(self.float_val <= other.float_val)
        assert False

    def gt(self, other):
        if isinstance(other, IntValue):
            return BoolValue(self.float_val > other.int_val)
        elif isinstance(other, FloatValue):
            return BoolValue(self.float_val > other.float_val)
        assert False

    def gte(self, other):
        if isinstance(other, IntValue):
            return BoolValue(self.float_val >= other.int_val)
        elif isinstance(other, FloatValue):
            return BoolValue(self.float_val >= other.float_val)
        assert False

    def min(self, other):
        if isinstance(other, IntValue):
            return FloatValue(min(self.float_val, float(other.int_val)))
        elif isinstance(other, FloatValue):
            return FloatValue(min(self.float_val, other.float_val))
        assert False

    def max(self, other):
        if isinstance(other, IntValue):
            return FloatValue(max(self.float_val, float(other.int_val)))
        elif isinstance(other, FloatValue):
            return FloatValue(max(self.float_val, other.float_val))
        assert False

    def is_truthy(self):
        return self.float_val != 0.0

    def to_number(self):
        return self


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

    def is_truthy(self):
        return self.str_val != ""

    def eq(self, other):
        if isinstance(other, StrValue):
            return BoolValue(self.str_val == other.str_val)
        return BoolValue(False)

    def to_number(self):
        try:
            if "." in self.str_val:
                return FloatValue(float(self.str_val))
            else:
                return IntValue(int(self.str_val))
        except:
            return None
