#ifndef LOL_OPS_H_INCLUDED
#define LOL_OPS_H_INCLUDED
#include "lol_runtime.h"
#define OP_FN(fn_name)                                                         \
  LolValue lol_##fn_name(LolValue left, LolValue right, LolSpan sp);

OP_FN(add)
OP_FN(sub)
OP_FN(mul)
OP_FN(div)
OP_FN(mod)

OP_FN(min)
OP_FN(max)

OP_FN(and)
OP_FN(or)
OP_FN(and)

OP_FN(eq)
OP_FN(neq)

OP_FN(lt)
OP_FN(lte)
OP_FN(gt)
OP_FN(gte)

LolValue lol_not(LolValue val);

LolValue to_lol_troof(LolValue value);
LolValue to_lol_numbr(LolValue value);
LolValue to_lol_numbar(LolValue value);
LolValue to_lol_yarn(LolValue value);

#endif
