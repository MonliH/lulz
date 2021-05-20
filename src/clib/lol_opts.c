#include "lol_runtime.h"
#include <stdio.h>

LolValue to_numeric(LolValue val) {
  if (IS_INT(val) || IS_DOUBLE(val))
    return val;
  else if (IS_BOOL(val))
    return INT_VALUE(val.as_u64);
  else if (IS_NULL(val))
    return INT_VALUE(0);
  else if (IS_FUN(val))
    exit(0);
}

#define BINARY_OP(op_name, op)                                                 \
  LolValue lol_##op_name(LolValue left, LolValue right, LolSpan sp) {          \
    LolValue l = to_numeric(left);                                             \
    LolValue r = to_numeric(right);                                            \
    if (IS_INT(l) && IS_INT(r)) {                                              \
      return INT_VALUE(AS_INT(l) op AS_INT(r));                                \
    } else if (IS_INT(l) && IS_DOUBLE(r)) {                                    \
      l = DOUBLE_VALUE((double)AS_INT(l));                                     \
    } else if (IS_DOUBLE(l) && IS_INT(r)) {                                    \
      r = DOUBLE_VALUE((double)AS_INT(r));                                     \
    }                                                                          \
    return DOUBLE_VALUE(AS_DOUBLE(l) op AS_DOUBLE(r));                         \
  }

BINARY_OP(add, +)
BINARY_OP(sub, -)
BINARY_OP(mul, *)
BINARY_OP(div, /)

LolValue lol_mod(LolValue left, LolValue right, LolSpan sp) {
  LolValue l = to_numeric(left);
  LolValue r = to_numeric(right);
  if (IS_INT(l) && IS_INT(r)) {
    return INT_VALUE(AS_INT(l) % AS_INT(r));
  } else {
    exit(1);
  }
}

#define BOOL_NUM_OP(op_name, op)                                               \
  LolValue lol_##op_name(LolValue left, LolValue right, LolSpan sp) {          \
    LolValue l = to_numeric(left);                                             \
    LolValue r = to_numeric(right);                                            \
    if (IS_INT(l) && IS_INT(r)) {                                              \
      return BOOL_VALUE(AS_INT(l) op AS_INT(r));                               \
    } else if (IS_INT(l) && IS_DOUBLE(r)) {                                    \
      l = DOUBLE_VALUE((double)AS_INT(l));                                     \
    } else if (IS_DOUBLE(l) && IS_INT(r)) {                                    \
      r = DOUBLE_VALUE((double)AS_INT(r));                                     \
    }                                                                          \
    return BOOL_VALUE(AS_DOUBLE(l) op AS_DOUBLE(r));                           \
  }

BOOL_NUM_OP(gt, >)
BOOL_NUM_OP(lt, <)
BOOL_NUM_OP(gte, >=)
BOOL_NUM_OP(lte, <=)
