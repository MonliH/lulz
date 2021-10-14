from rpython.rlib.objectmodel import instantiate

class Value:
    def get_value(self):
        return self

    def __deepcopy__(self):
        return instantiate(self.__class__)

    def str(self):
        return ""

    def add(self, other):
        return self

class IntValue(Value):
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
            return IntValue(self.int_val + other.float_val)
        assert False

class FloatValue(Value):
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
