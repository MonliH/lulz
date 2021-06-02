#include "lol_runtime.h"
#include <errno.h>
#include <stdarg.h>
#include <stdio.h>
#include <string.h>

LolValue to_numeric(LolValue val, LolSpan sp) {
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
        printf("invalid number\n");
        exit(1);
      }
      return DOUBLE_VALUE(value);
    } else {
      char *e;
      errno = 0;
      int32_t value = strtol(str, &e, 10);
      if (*e != '\0' || errno != 0) {
        printf("invalid number\n");
        exit(1);
      }
      return INT_VALUE(value);
    }
  }
  printf("invalid number %d:%d\n", sp.s, sp.e);
  exit(1);
}

#define BINARY_OP(op_name, op)                                                 \
  LolValue lol_##op_name(LolValue left, LolValue right, LolSpan sp) {          \
    LolValue l = to_numeric(left, sp);                                         \
    LolValue r = to_numeric(right, sp);                                        \
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
  LolValue l = to_numeric(left, sp);
  LolValue r = to_numeric(right, sp);
  if (IS_INT(l) && IS_INT(r)) {
    return INT_VALUE(AS_INT(l) % AS_INT(r));
  } else {
    exit(1);
  }
}

#define BOOL_NUM_OP(op_name, op)                                               \
  LolValue lol_##op_name(LolValue left, LolValue right, LolSpan sp) {          \
    LolValue l = to_numeric(left, sp);                                         \
    LolValue r = to_numeric(right, sp);                                        \
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

LolValue to_lol_numbar(LolValue value, LolSpan sp) {
  LolValue num = to_numeric(value, sp);
  if (IS_INT(num)) {
    num = DOUBLE_VALUE((double)(AS_INT(num)));
  }
  return num;
}

LolValue to_lol_numbr(LolValue value, LolSpan sp) {
  LolValue num = to_numeric(value, sp);
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
  LolValue lol_##name(LolValue left, LolValue right, LolSpan sp) {             \
    LolValue l = to_numeric(left, sp);                                         \
    LolValue r = to_numeric(right, sp);                                        \
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

LolValue lol_not(LolValue value, LolSpan sp) {
  bool b = lol_to_bool(value);
  return BOOL_VALUE(!b);
}

LolValue lol_length(LolValue value, LolSpan sp) {
  if (IS_STR(value)) {
    return INT_VALUE(AS_STR(value)->len);
  } else if (IS_VEC(value)) {
    return INT_VALUE(AS_VEC(value)->len);
  } else {
    printf("could not get length: not a string or list\n");
    exit(1);
  }
}

#define UNARY_MATH(name, intval, floatval)                                     \
  LolValue lol_##name(LolValue value, LolSpan sp) {                            \
    LolValue n = to_numeric(value, sp);                                            \
    if (IS_INT(n)) {                                                           \
      int32_t num = AS_INT(n);                                                 \
      return INT_VALUE(num intval);                                            \
    } else if (IS_DOUBLE(n)) {                                                 \
      double num = AS_DOUBLE(n);                                               \
      return DOUBLE_VALUE(num floatval);                                       \
    }                                                                          \
  }

UNARY_MATH(uppin, +1, +1.0)
UNARY_MATH(nerfin, -1, -1.0)

#define BOOL_OP(name, cmp)                                                     \
  LolValue lol_##name(LolValue left, LolValue right) {                         \
    bool l = lol_to_bool(left);                                                \
    bool r = lol_to_bool(right);                                               \
    return BOOL_VALUE((l cmp r));                                              \
  }

BOOL_OP(and, &&)
BOOL_OP(or, ||)
BOOL_OP(xor, !=)

LolValue lol_any(size_t length, ...) {
  va_list args;
  va_start(args, length);
  for (size_t i = 0; i < length; i++) {
    LolValue boolish = va_arg(args, LolValue);
    if (lol_to_bool(boolish)) {
      return TRUE_VALUE;
    }
  }
  va_end(args);
  return FALSE_VALUE;
}

LolValue lol_all(size_t length, ...) {
  va_list args;
  va_start(args, length);
  for (size_t i = 0; i < length; i++) {
    LolValue boolish = va_arg(args, LolValue);
    if (!lol_to_bool(boolish)) {
      return FALSE_VALUE;
    }
  }
  va_end(args);
  return TRUE_VALUE;
}

void lol_append(LolValue source, LolValue item, LolSpan sp) {
  if (IS_VEC(source)) {
    VectorObj *vec = AS_VEC(source);
    lol_vec_append(vec, item);
  } else {
    printf("could not append: not a list\n");
    exit(1);
  }
}

LolValue lol_vec_index(LolValue source, LolValue idx, LolSpan sp) {
  if (IS_VEC(source)) {
    VectorObj *vec = AS_VEC(source);
    if (IS_INT(idx)) {
      int32_t i = AS_INT(idx);
      if (vec->len > i && i >= 0) {
        return vec->items[i];
      } else {
        printf("get: index out of range\n");
        exit(1);
      }
    } else {
      printf("get: index not an int %d:%d\n", sp.s, sp.e);
      exit(1);
    }
  } else {
    printf("get: array not an array %d:%d\n", sp.s, sp.e);
    exit(1);
  }
}

LolValue lol_vec_first(LolValue source, LolSpan sp) {
  if (IS_VEC(source)) {
    VectorObj *vec = AS_VEC(source);
    if (vec->len > 0) {
      return vec->items[0];
    } else {
      printf("get: index out of range\n");
      exit(1);
    }
  } else {
    printf("get: array not an array\n");
    exit(1);
  }
}

LolValue lol_vec_last(LolValue source, LolSpan sp) {
  if (IS_VEC(source)) {
    VectorObj *vec = AS_VEC(source);
    if (vec->len > 0) {
      return vec->items[vec->len - 1];
    } else {
      printf("get: index out of range\n");
      exit(1);
    }
  } else {
    printf("get: array not an array\n");
    exit(1);
  }
}

LolValue lol_vec_set(LolValue source, LolValue idx, LolValue value,
                     LolSpan sp) {
  if (IS_VEC(source)) {
    VectorObj *vec = AS_VEC(source);
    if (IS_INT(idx)) {
      int32_t i = AS_INT(idx);
      if (vec->len > i && i >= 0) {
        vec->items[i] = value;
      } else {
        printf("set: index out of range\n");
        exit(1);
      }
    } else {
      printf("set: index not an int %d:%d\n", sp.s, sp.e);
      exit(1);
    }
  } else {
    printf("set: array not an array\n");
    exit(1);
  }
}

LolValue lol_vec_set_first(LolValue source, LolValue value, LolSpan sp) {
  if (IS_VEC(source)) {
    VectorObj *vec = AS_VEC(source);
    if (vec->len > 0) {
      vec->items[0] = value;
    } else {
      printf("set: index out of range\n");
      exit(1);
    }
  } else {
    printf("set: array not an array\n");
    exit(1);
  }
}

LolValue lol_vec_set_last(LolValue source, LolValue value, LolSpan sp) {
  if (IS_VEC(source)) {
    VectorObj *vec = AS_VEC(source);
    if (vec->len > 0) {
      vec->items[vec->len - 1] = value;
    } else {
      printf("set: index out of range\n");
      exit(1);
    }
  } else {
    printf("set: array not an array\n");
    exit(1);
  }
}
