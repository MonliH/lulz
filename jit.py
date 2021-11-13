import sys
from rpython import conftest


class o:
    view = True
    viewloops = True
conftest.option = o

from rpython.rlib.nonconst import NonConstant
from rpython.rlib import jit
from rpython.jit.metainterp.test.test_ajit import LLJitMixin
from vm import init_vm_from_source


class TestLLtype(LLJitMixin):
    def run_string(self, source):
        vm = init_vm_from_source(source)
        if not vm:
            raise RuntimeError("Compile Error")

        def interp_w(vm):
            def interp():
                jit.set_param(None, "disable_unrolling", 5000)
                vm.interpret()
            return interp

        interp_w(vm)()  # check that it runs
        vm = init_vm_from_source(source)

        self.meta_interp(interp_w(vm), [], listcomp=True, listops=True, backendopt=True, inline=True)

    def test_recursive(self):
        self.run_string("""HAI 1.3

BTW THIS IS THE SLOW FIB FUNCTION, WITH EXPONENTIAL RUNTIME
HOW IZ I FIB YR N
    IZ N LES EQ THEN 1
    O RLY?
        YA RLY
            FOUND YR 1
        NO WAI
            FOUND YR SUM OF I IZ FIB YR DIFF OF N AN 1 MKAY AN I IZ FIB YR DIFF OF N AN 2 MKAY
    OIC
IF U SAY SO

VISIBLE I IZ FIB YR 10 MKAY
KTHXBYE
""")
