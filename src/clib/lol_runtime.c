#include "lol_runtime.h"

LolValue lol_it = ((LolValue){LOL_NOOB, {.numbr = 0}});

LolValue lol_call(unsigned short args, LolValue fn, struct LolValue *values) {
  if (fn.type != LOL_FUNKSHON) {
    exit(1);
  }

  Function func = fn.as.funkshon;
  if (func.args != args) {
    exit(1);
  }

  return func.fn(args, values);
}
