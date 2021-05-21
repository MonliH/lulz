#include "lol_runtime.h"
#include <inttypes.h>
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

char *lol_to_str(LolValue value) {
  if (IS_INT(value)) {
    int i = AS_INT(value);
    int length = snprintf(NULL, 0, "%" PRId32 "", i);
    char *str = malloc(length + 1);
    snprintf(str, length + 1, "%" PRId32 "", i);
    return str;
  } else if (IS_DOUBLE(value)) {
    double dbl = AS_DOUBLE(value);
    int length = snprintf(NULL, 0, "%g", dbl);
    char *str = malloc(length + 1);
    snprintf(str, length + 1, "%g", dbl);
    return str;
  } else if (IS_BOOL(value)) {
    return AS_BOOL(value) ? "WIN" : "FAIL";
  } else if (IS_NULL(value)) {
    return "NOOB";
  } else if (IS_FUN(value)) {
    uint64_t fn_id = (uint64_t)AS_FUN(value);
    char format_str[] = "<FUNKSHON at 0x%08lx>";
    int length = snprintf(NULL, 0, format_str, fn_id);
    char *str = malloc(length + 1);
    snprintf(str, length + 1, format_str, fn_id);
    return str;
  } else if (IS_STR(value)) {
    return AS_CSTR(value);
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

Obj *lol_allocate_obj(size_t size, ObjType type) {
  Obj *object = (Obj *)lol_reallocate(NULL, 0, size);
  object->ty = type;
  return object;
}

StringObj *lol_allocate_lit_str(char *chars, int length) {
  StringObj *string = ALLOCATE_OBJ(StringObj, OBJ_STRING);
  string->len = length;
  char *mchars = malloc(length);
  strncpy(mchars, chars, length);
  string->chars = mchars;
  return string;
}

StringObj *lol_allocate_str(char *chars, int length) {
  StringObj *string = ALLOCATE_OBJ(StringObj, OBJ_STRING);
  string->len = length;
  string->chars = chars;
  return string;
}

void *lol_reallocate(void *pointer, size_t oldSize, size_t newSize) {
  if (newSize == 0) {
    free(pointer);
    return NULL;
  }

  void *result = realloc(pointer, newSize);
  if (result == NULL)
    exit(1);
  return result;
}
