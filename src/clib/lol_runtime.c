#include "lol_runtime.h"
#include <inttypes.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

LolValue lol_it = NULL_VALUE;

LolValue lol_call(uint8_t args, LolValue fn, LolValue *values, LolSpan sp) {
  if (!IS_FUN(fn)) {
    exit(1);
  }

  LolFn func = AS_FUN(fn);

  return func(args, values);
}

StringObj lol_to_str(LolValue value) {
  if (IS_INT(value)) {
    int32_t i = AS_INT(value);
    size_t length = snprintf(NULL, 0, "%" PRId32 "", i);
    char *str = ALLOCATE(char, length + 1);
    snprintf(str, length + 1, "%" PRId32 "", i);
    return MAKE_STR_OBJ(str, length, false);
  } else if (IS_DOUBLE(value)) {
    double dbl = AS_DOUBLE(value);
    size_t length = snprintf(NULL, 0, "%g", dbl);
    char *str = ALLOCATE(char, length + 1);
    snprintf(str, length + 1, "%g", dbl);
    return MAKE_STR_OBJ(str, length, false);
  } else if (IS_BOOL(value)) {
    return AS_BOOL(value) ? MAKE_STR_OBJ("WIN", 3, true)
                          : MAKE_STR_OBJ("FAIL", 4, true);
  } else if (IS_NULL(value)) {
    return MAKE_STR_OBJ("NOOB", 4, true);
  } else if (IS_FUN(value)) {
    uint64_t fn_id = (uint64_t)AS_FUN(value);
    char format_str[] = "<FUNKSHON at 0x%08lx>";
    size_t length = snprintf(NULL, 0, format_str, fn_id);
    char *str = ALLOCATE(char, length + 1);
    snprintf(str, length + 1, format_str, fn_id);
    return MAKE_STR_OBJ(str, length, false);
  } else if (IS_STR(value)) {
    return *AS_STR(value);
  }
}

size_t lol_str_len(LolValue value) {
  if (IS_INT(value)) {
    int32_t i = AS_INT(value);
    size_t length = snprintf(NULL, 0, "%" PRId32 "", i);
    return length;
  } else if (IS_DOUBLE(value)) {
    double dbl = AS_DOUBLE(value);
    size_t length = snprintf(NULL, 0, "%g", dbl);
    return length;
  } else if (IS_BOOL(value)) {
    return AS_BOOL(value) ? 3 : 4;
  } else if (IS_NULL(value)) {
    return 4;
  } else if (IS_FUN(value)) {
    uint64_t fn_id = (uint64_t)AS_FUN(value);
    char format_str[] = "<FUNKSHON at 0x%08lx>";
    size_t length = snprintf(NULL, 0, format_str, fn_id);
    return length;
  } else if (IS_STR(value)) {
    return AS_STR(value)->len;
  }
}

void lol_print(LolValue value) {
  if (IS_DOUBLE(value)) {
    printf("%g", AS_DOUBLE(value));
  } else if (IS_INT(value)) {
    printf("%" PRId32 "", AS_INT(value));
  } else if (IS_BOOL(value)) {
    printf("%s", AS_BOOL(value) ? "WIN" : "FAIL");
  } else if (IS_NULL(value)) {
    printf("NOOB");
  } else if (IS_FUN(value)) {
    printf("<FUNKSHON at 0x%08lx>", (uint64_t)AS_FUN(value));
  } else if (IS_STR(value)) {
    printf("%s", AS_CSTR(value));
  }
}

bool lol_to_bool(LolValue value) {
  if (IS_BOOL(value)) {
    return (AS_BOOL(value));
  } else if (IS_INT(value)) {
    return (AS_INT(value)) != 0;
  } else if (IS_DOUBLE(value)) {
    return (AS_DOUBLE(value)) != 0.0;
  } else if (IS_NULL(value)) {
    return false;
  } else if (IS_STR(value)) {
    return AS_CSTR(value)[0] != '\0';
  } else if (IS_FUN(value)) {
    return true;
  } else {
    printf("internal error lol_to_bool\n");
    exit(1);
  }
}

bool lol_is_obj_ty(LolValue value, ObjType ty) {
  return IS_OBJ(value) && AS_OBJ(value)->ty == ty;
}

void lol_println(LolValue value) {
  lol_print(value);
  printf("\n");
}

Obj *lol_alloc_obj(size_t size, ObjType type, bool constant) {
  Obj *object = (Obj *)lol_realloc(NULL, 0, size);
  object->ty = type;
  object->constant = constant;
  return object;
}

StringObj *lol_alloc_lit_str(char *chars, int length) {
  StringObj *string = ALLOCATE_OBJ(StringObj, OBJ_STRING, true);
  string->len = length;
  char *mchars = ALLOCATE(char, length);
  strncpy(mchars, chars, length);
  string->chars = mchars;
  return string;
}

StringObj *lol_alloc_str(char *chars, int length) {
  StringObj *string = ALLOCATE_OBJ(StringObj, OBJ_STRING, false);
  string->len = length;
  string->chars = chars;
  return string;
}

void *lol_realloc(void *pointer, size_t oldSize, size_t newSize) {
  if (newSize == 0) {
    free(pointer);
    return NULL;
  }

  void *result = realloc(pointer, newSize);
  if (result == NULL)
    exit(1);
  return result;
}

StringObj *lol_alloc_stack_str(StringObj obj) {
  StringObj *o = (StringObj *)lol_alloc_obj(sizeof(StringObj), obj.obj.ty,
                                            obj.obj.constant);
  *o = obj;
  return o;
}

StringObj lol_concat_str(size_t length, ...) {
  size_t str_lens = 1;
  va_list args;
  va_start(args, length);
  for (size_t i = 0; i < length; i++) {
    LolValue v = va_arg(args, LolValue);
    str_lens += lol_str_len(v);
  }
  va_end(args);
  char *final_str = lol_realloc(NULL, 0, str_lens + 1);
  char *end = final_str;

  va_start(args, length);
  for (size_t i = 0; i < length; i++) {
    LolValue v = va_arg(args, LolValue);
    StringObj s = lol_to_str(v);
    end = stpncpy(end, s.chars, s.len);
  }
  va_end(args);

  final_str[str_lens] = '\0';

  return MAKE_STR_OBJ(final_str, str_lens, false);
}

void lol_readline(LolValue *val) {
  size_t n = 0, result;
  char *buf;

  result = getline(&buf, &n, stdin);
  if (result < 0)
    exit(1);

  size_t len = strlen(buf);
  if (len > 0 && buf[len - 1] == '\n') {
    buf[--len] = '\0';
  }

  *val = OBJ_VALUE(lol_alloc_str(buf, len));
}
