#include "lol_runtime.h"
#include <inttypes.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

char *stpncpy(char *dst, const char *src, size_t len) {
  size_t n = strlen(src);
  if (n > len)
    n = len;
  return strncpy(dst, src, len) + n;
}

LolValue lol_it = NULL_VALUE;

LolValue lol_call(uint8_t args, LolValue fn, LolValue *values, LolSpan sp) {
  if (IS_FUN(fn)) {
    LolFn func = AS_FUN(fn);

    return func(args, values);
  } else if (IS_CLOSURE(fn)) {
    ClosureObj *closure = AS_CLOSURE(fn);
    return closure->fn(args, values, closure->upvalues);
  } else {
    exit(1);
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
  } else if (IS_VEC(value)) {
    VectorObj *vec = AS_VEC(value);
    // "[]"
    size_t len = 2;
    if (vec->len > 0) {
      // ", "
      len += (vec->len - 1) * 2;
    }
    for (size_t i = 0; i < vec->len; i++) {
      len += lol_str_len(vec->items[i]);
    }
    return len;
  }
}

StringObj lol_to_str(LolValue value) {
  size_t length = lol_str_len(value);
  if (IS_INT(value)) {
    char *str = ALLOCATE(char, length + 1);
    snprintf(str, length + 1, "%" PRId32 "", AS_INT(value));
    return MAKE_STR_OBJ(str, length, false);
  } else if (IS_DOUBLE(value)) {
    char *str = ALLOCATE(char, length + 1);
    snprintf(str, length + 1, "%g", AS_DOUBLE(value));
    return MAKE_STR_OBJ(str, length, false);
  } else if (IS_BOOL(value)) {
    return AS_BOOL(value) ? MAKE_STR_OBJ("WIN", 3, true)
                          : MAKE_STR_OBJ("FAIL", 4, true);
  } else if (IS_NULL(value)) {
    return MAKE_STR_OBJ("NOOB", 4, true);
  } else if (IS_FUN(value)) {
    char *str = ALLOCATE(char, length + 1);
    snprintf(str, length + 1, "<FUNKSHON at 0x%08lx>", (uint64_t)AS_FUN(value));
    return MAKE_STR_OBJ(str, length, false);
  } else if (IS_STR(value)) {
    return *AS_STR(value);
  } else if (IS_VEC(value)) {
    VectorObj *vec = AS_VEC(value);
    char *str = ALLOCATE(char, length + 1);
    str[0] = '[';
    char *end = str + sizeof(char);
    if (vec->len > 0) {
      StringObj s = lol_to_str(vec->items[0]);
      end = stpncpy(end, s.chars, s.len);
      for (size_t i = 1; i < vec->len; i++) {
        end = stpncpy(end, ", ", 2);
        StringObj s = lol_to_str(vec->items[i]);
        end = stpncpy(end, s.chars, s.len);
      }
    }
    end[0] = ']';
    end[1] = '\0';
    return MAKE_STR_OBJ(str, length, false);
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
  char *mchars = ALLOCATE(char, length + 1);
  strncpy(mchars, chars, length + 1);
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
  if (result == NULL) {
    exit(1);
  }
  return result;
}

StringObj *lol_alloc_stack_str(StringObj obj) {
  StringObj *o = (StringObj *)lol_alloc_obj(sizeof(StringObj), obj.obj.ty,
                                            obj.obj.constant);
  *o = obj;
  return o;
}

StringObj lol_concat_str(size_t length, ...) {
  size_t str_lens = 0;
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

StringObj lol_interp_str(size_t length, ...) {
  size_t str_lens = 0;
  va_list args;
  va_start(args, length);
  for (size_t i = 0; i < length; i++) {
    size_t v = va_arg(args, size_t);
    str_lens += v;
    va_arg(args, char *);
    str_lens += lol_str_len(va_arg(args, LolValue));
  }
  // last string (extra at end)
  str_lens += va_arg(args, size_t);
  va_arg(args, char *);
  va_end(args);

  char *final_str = lol_realloc(NULL, 0, str_lens + 1);
  char *end = final_str;

  va_start(args, length);
  for (size_t i = 0; i < length; i++) {
    size_t lit_len = va_arg(args, size_t);
    end = stpncpy(end, va_arg(args, char *), lit_len);

    LolValue v = va_arg(args, LolValue);
    StringObj s = lol_to_str(v);
    end = stpncpy(end, s.chars, s.len);
  }
  size_t lit_len = va_arg(args, size_t);
  end = stpncpy(end, va_arg(args, char *), lit_len);
  va_end(args);

  final_str[str_lens] = '\0';

  return MAKE_STR_OBJ(final_str, str_lens, false);
}

VectorObj *lol_alloc_stack_vec(VectorObj obj) {
  VectorObj *o = (VectorObj *)lol_alloc_obj(sizeof(VectorObj), obj.obj.ty,
                                            obj.obj.constant);
  *o = obj;
  return o;
}

VectorObj lol_init_vec() {
  return (VectorObj){(Obj){OBJ_VECTOR, false}, 0, 0, NULL};
}

size_t grow_cap(size_t cap) { return (cap < 8) ? 8 : (cap * 2); }

void lol_vec_capacity(VectorObj *vec, size_t new_size) {
#ifdef LOL_DEBUG_CHECK
  if (vec->cap > new_size) {
    printf("internal error lol_vec_capacity\n");
    exit(1);
  }
#endif
  size_t old_cap = vec->cap;
  vec->cap = new_size;
  vec->items = GROW_VEC(LolValue, vec->items, old_cap, vec->cap);
}

void lol_vec_append(VectorObj *vec, LolValue val) {
  if (vec->cap < (vec->len + 1)) {
    lol_vec_capacity(vec, grow_cap(vec->cap));
  }

  vec->items[vec->len] = val;
  vec->len++;
}

LolValue lol_vec_lit(size_t cap, size_t length, ...) {
  VectorObj *vec = lol_alloc_stack_vec(lol_init_vec());
  lol_vec_capacity(vec, cap);

  va_list args;
  va_start(args, length);

  for (size_t i = 0; i < length; i++) {
    lol_vec_append(vec, va_arg(args, LolValue));
  }
  va_end(args);

  return OBJ_VALUE((Obj *)vec);
}

ClosureObj lol_init_closure(LolClosureFn fn, size_t upvalue_count, ...) {
  DynPtrObj **upvalues = ALLOCATE(DynPtrObj *, upvalue_count);
  va_list args;
  va_start(args, upvalue_count);

  for (size_t i = 0; i < upvalue_count; i++) {
    upvalues[i] = va_arg(args, DynPtrObj*);
  }
  va_end(args);

  return (ClosureObj){(Obj){OBJ_CLOSURE, false}, fn, upvalues, upvalue_count};
}

ClosureObj *lol_alloc_stack_closure(ClosureObj obj) {
  ClosureObj *o = (ClosureObj *)lol_alloc_obj(sizeof(ClosureObj), obj.obj.ty,
                                              obj.obj.constant);
  *o = obj;
  return o;
}

DynPtrObj lol_init_dyn_ptr(LolValue* ptr) {
  return (DynPtrObj){(Obj){OBJ_PTR, false}, ptr};
}

DynPtrObj *lol_alloc_stack_dyn_ptr(DynPtrObj obj) {
  DynPtrObj *o = (DynPtrObj *)lol_alloc_obj(sizeof(DynPtrObj), obj.obj.ty,
                                              obj.obj.constant);
  *o = obj;
  return o;
}

void lol_box_dyn_ptr(DynPtrObj *ptr) {
  LolValue cpy = *ptr->ptr;
  LolValue* new_loc = lol_realloc(NULL, 0, sizeof(LolValue));
  *new_loc = cpy;
  ptr->ptr = new_loc;
}
