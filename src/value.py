from rpython.rlib.objectmodel import instantiate


class Value:
    __slots__ = ()

    def get_value(self):
        return self

    def __deepcopy__(self):
        return instantiate(self.__class__)

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


class BoolValue(Value):
    __slots__ = ("bool_val",)
    _immutable_fields_ = ["bool_val"]

    def __init__(self, bool_val):
        assert isinstance(bool_val, bool)
        self.bool_val = bool_val

    def str(self):
        return "WIN" if self.bool_val else "FAIL"

    def add(self, other):
        return self

    def sub(self, other):
        return self

    def mul(self, other):
        return self

    def div(self, other):
        return self

    def eq(self, other):
        if isinstance(other, BoolValue):
            return BoolValue(self.bool_val == other.bool_val)
        return BoolValue(False)

    def is_truthy(self):
        return self.bool_val


class NullValue(Value):
    __slots__ = ()
    _immutable_fields_ = []

    def __init__(self):
        pass

    def str(self):
        return "NOOB"

    def add(self, other):
        return self

    def sub(self, other):
        return self

    def mul(self, other):
        return self

    def div(self, other):
        return self

    def eq(self, other):
        if isinstance(other, NullValue):
            return BoolValue(True)
        return BoolValue(False)

    def is_truthy(self):
        return False


class IntValue(Value):
    __slots__ = ("int_val",)
    _immutable_fields_ = ["int_val"]

    def __init__(self, int_val):
        assert isinstance(int_val, int)
        self.int_val = int_val

    def str(self):
        return str(self.int_val)

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

    def eq(self, other):
        if isinstance(other, IntValue):
            return BoolValue(self.int_val == other.int_val)
        return BoolValue(False)

    def is_truthy(self):
        return self.int_val != 0


class FloatValue(Value):
    __slots__ = ("float_val",)
    _immutable_fields_ = ["float_val"]

    def __init__(self, float_val):
        assert isinstance(float_val, float)
        self.float_val = float_val

    def str(self):
        return str(self.float_val)

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
        return BoolValue(False)

    def is_truthy(self):
        return self.float_val != 0.0


class StrValue(Value):
    __slots__ = ("str_val",)
    _immutable_fields_ = ["str_val"]

    def __init__(self, str_val):
        assert isinstance(str_val, str)
        self.str_val = str_val

    def str(self):
        return self.str_val

    def is_truthy(self):
        return self.str_val != ""

    def eq(self, other):
        if isinstance(other, StrValue):
            return BoolValue(self.str_val == other.str_val)
        return BoolValue(False)
