from rpython.rlib.objectmodel import instantiate

class Value(object):
    def get_value(self):
        return self

    def __deepcopy__(self):
        return instantiate(self.__class__)

    def str(self):
        raise NotImplementedError("str not implemented")

class IntValue(Value):
    _immutable_fields_ = ["int_val"]

    def __init__(self, int_val):
        self.int_val = int_val

    def str(self):
        return str(self.int_val)
