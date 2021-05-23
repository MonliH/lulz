#include "lol_runtime.h"
#include <errno.h>
#include <stdio.h>
#include <string.h>

LolValue to_numeric(LolValue val) {
  if (IS_INT(val) || IS_DOUBLE(val))
    return val;
  else if (IS_BOOL(val))
    return INT_VALUE(val.as_u64);
  else if (IS_NULL(val))
    return INT_VALUE(0);
  else if (IS_STR(val)) {
    char *str = AS_CSTR(val);
    bool has_decimal = strchr(str, '.') != NULL;
    if (has_decimal) {
      char *e;
      errno = 0;
      double value = strtod(str, &e);
      if (*e != '\0' || errno != 0) {
        exit(1);
      }
      return DOUBLE_VALUE(value);
    } else {
      char *e;
      errno = 0;
      int32_t value = strtol(str, &e, 10);
      if (*e != '\0' || errno != 0) {
        exit(1);
      }
      return INT_VALUE(value);
    }
  }
  exit(1);
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

#define EQ_OP(op_name, op, t, f)                                               \
  LolValue lol_##op_name(LolValue l, LolValue r, LolSpan sp) {                 \
    if (IS_INT(l) && IS_INT(r))                                                \
      return BOOL_VALUE(AS_INT(l) op AS_INT(r));                               \
    else if (IS_DOUBLE(l) && IS_DOUBLE(r))                                     \
      return BOOL_VALUE(AS_DOUBLE(l) op AS_DOUBLE(r));                         \
    else if (IS_DOUBLE(l) && IS_INT(r))                                        \
      return BOOL_VALUE(AS_DOUBLE(l) op(double) AS_INT(r));                    \
    else if (IS_INT(l) && IS_DOUBLE(r))                                        \
      return BOOL_VALUE((double)AS_INT(l) op AS_DOUBLE(r));                    \
    else if (IS_BOOL(l) && IS_BOOL(r))                                         \
      return BOOL_VALUE(AS_BOOL(l) op AS_BOOL(r));                             \
    else if (IS_FUN(l) && IS_FUN(r))                                           \
      return BOOL_VALUE(AS_FUN(l) op AS_FUN(r));                               \
    else if (IS_NULL(l) && IS_NULL(r)) {                                       \
      return BOOL_VALUE(t);                                                    \
    } else if (IS_STR(l) && IS_STR(r)) {                                       \
      return BOOL_VALUE(strcmp(AS_CSTR(l), AS_CSTR(r)) op 0);                  \
    }                                                                          \
    return BOOL_VALUE(f);                                                      \
  }

EQ_OP(eq, ==, true, false)
EQ_OP(neq, !=, false, true)

LolValue to_lol_troof(LolValue value) { return BOOL_VALUE(lol_to_bool(value)); }

LolValue to_lol_numbar(LolValue value) {
  LolValue num = to_numeric(value);
  if (IS_INT(num)) {
    num = DOUBLE_VALUE((double)(AS_INT(num)));
  }
  return num;
}

LolValue to_lol_numbr(LolValue value) {
  LolValue num = to_numeric(value);
  if (IS_DOUBLE(num)) {
    num = INT_VALUE((int64_t)(AS_DOUBLE(num)));
  }
  return num;
}

LolValue to_lol_yarn(LolValue value) {
  StringObj *yarn = lol_alloc_stack_str(lol_to_str(value));
  return OBJ_VALUE(yarn);
}

#define CMP_OP(name, cmp)                                                      \
  LolValue lol_##name(LolValue left, LolValue right) {                         \
    LolValue l = to_numeric(left);                                             \
    LolValue r = to_numeric(right);                                            \
    if (IS_INT(l) && IS_INT(r)) {                                              \
      int32_t cl = AS_INT(l);                                                  \
      int32_t cr = AS_INT(r);                                                  \
      return (cmp) ? l : r;                                                    \
    } else if (IS_INT(l) && IS_DOUBLE(r)) {                                    \
      l = DOUBLE_VALUE((double)AS_INT(l));                                     \
    } else if (IS_DOUBLE(l) && IS_INT(r)) {                                    \
      r = DOUBLE_VALUE((double)AS_INT(r));                                     \
    }                                                                          \
    double cl = AS_DOUBLE(l);                                                  \
    double cr = AS_DOUBLE(r);                                                  \
    return (cmp) ? l : r;                                                      \
  }

CMP_OP(min, (cl < cr))
CMP_OP(max, (cl > cr))
