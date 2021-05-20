#ifndef LOL_OPTS_H_INCLUDED
#define LOL_OPTS_H_INCLUDED
#include "lol_runtime.h"
#define OPT_FN(fn_name) LolValue lol_ ## fn_name(LolValue left, LolValue right, LolSpan sp);

OPT_FN(add)
OPT_FN(sub)
OPT_FN(mul)
OPT_FN(div)
OPT_FN(mod)

OPT_FN(min)
OPT_FN(max)

OPT_FN(and)
OPT_FN(or)
OPT_FN(and)

OPT_FN(eq)

OPT_FN(lt)
OPT_FN(lte)
OPT_FN(gt)
OPT_FN(gte)

#endif
