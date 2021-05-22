#ifndef LOL_RUNTIME_H_INCLUDED
#define LOL_RUNTIME_H_INCLUDED
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum {
  LOL_TROOF,
  LOL_NOOB,
  LOL_NUMBAR,
  LOL_NUMBR,
  LOL_FUNKSHON,
} LolValTy;

typedef union {
  uint64_t as_u64;
  double as_f64;
} LolValue;
typedef LolValue (*LolFn)(uint8_t args, LolValue *values);

// NAN-boxing is sort of crazy, I know
// AKA, 0 1111 1111 1111 100000000000000000000000000000000000000000000000000
#define NAN_VALUE 0x7ffc000000000000
// AKA, 1 1111 1111 1111 110000000000000000000000000000000000000000000000000
#define NAN_MASK 0xffff000000000000
// AKA, 0 1111 1111 1111 110000000000000000000000000000000000000000000000100
#define BOOLEAN_MASK 0x7ffe000000000002
// AKA, 0 1111 1111 1111 100000000000000000000000000000000000000000000000000
#define INTEGER_MASK 0x7ffc000000000000

#define TRUE_BITS (BOOLEAN_MASK | 3)
#define FALSE_BITS (BOOLEAN_MASK | 2)
// AKA, 0 1111 1111 1111 110000000000000000000000000000000000000000000000000
#define NULL_BITS 0x7ffe000000000000
#define NULL_VALUE ((LolValue){.as_u64 = ((uint64_t)(NULL_BITS))})

#define IS_DOUBLE(v) (((v).as_u64 & NAN_VALUE) != NAN_VALUE)
#define AS_DOUBLE(v) ((v).as_f64)
#define IS_NULL(v) ((v).as_u64 == NULL_BITS)
#define IS_BOOL(v) (((v).as_u64 & BOOLEAN_MASK) == BOOLEAN_MASK)
#define IS_TRUE(v) ((v).as_u64 == TRUE_BITS)
#define IS_FALSE(v) ((v).as_u64 == FALSE_BITS)
#define IS_INT(v) (((v).as_u64 & NAN_MASK) == INTEGER_MASK)

#define AS_BOOL(v) ((bool)((v).as_u64 & 0x1))
#define AS_INT(v) ((int32_t)((v).as_u64))

// AKA, 1 1111 1111 1111 100000000000000000000000000000000000000000000000000
#define FUN_MASK 0xfffc000000000000
// AKA, 0 0000 0000 0000 000111111111111111111111111111111111111111111111111
// (48 of 1's)
#define AS_FUN(v) ((LolFn)((v).as_u64 & 0xFFFFFFFFFFFF))
#define IS_FUN(v) (((v).as_u64 & NAN_MASK) == FUN_MASK)

// AKA, 1 1111 1111 1111 110000000000000000000000000000000000000000000000000
#define OBJ_MASK 0xfffe000000000000
// AKA, 0 0000 0000 0000 000111111111111111111111111111111111111111111111111
// (48 of 1's)
#define AS_OBJ(v) ((Obj *)((v).as_u64 & 0xFFFFFFFFFFFF))
#define IS_OBJ(v) (((v).as_u64 & NAN_MASK) == OBJ_MASK)

#define INT_VALUE(i) ((LolValue){.as_u64 = ((uint64_t)(i) | INTEGER_MASK)})
#define FUN_VALUE(f) ((LolValue){.as_u64 = ((uint64_t)(f) | FUN_MASK)})
#define OBJ_VALUE(o) ((LolValue){.as_u64 = ((uint64_t)(o) | OBJ_MASK)})
#define DOUBLE_VALUE(d) ((LolValue){.as_f64 = (d)})
#define BOOL_VALUE(b) ((LolValue){.as_u64 = ((b) ? TRUE_BITS : FALSE_BITS)})

#define OBJ_TYPE(value) (OBJ_VALUE(value)->ty)

#define IS_STR(value) lol_is_obj_ty(value, OBJ_STRING)
#define AS_STR(value) ((StringObj *)AS_OBJ(value))
#define AS_CSTR(value) (((StringObj *)AS_OBJ(value))->chars)

#define ALLOCATE(ty, count) (ty *)(lol_realloc(NULL, 0, sizeof(ty) * (count)))

#define ALLOCATE_OBJ(ty, objectType, constant)                                 \
  (ty *)(lol_alloc_obj(sizeof(ty), objectType, constant))

#define MAKE_STR_OBJ(str, len, constant)                                       \
  (StringObj) { OBJ_STRING, constant, (len), (str) }

typedef enum {
  OBJ_STRING,
} ObjType;

typedef struct {
  ObjType ty;
  bool constant;
} Obj;

typedef struct {
  Obj obj;
  size_t len;
  char *chars;
} StringObj;

typedef struct {
  uint32_t s;
  uint32_t e;
} LolSpan;

extern LolValue lol_it;

LolValue lol_call(uint8_t args, LolValue fn, LolValue *values, LolSpan sp);

void lol_print(LolValue value);
void lol_println(LolValue value);
bool lol_to_bool(LolValue value);
bool lol_is_obj_ty(LolValue, ObjType);

void *lol_realloc(void *pointer, size_t oldSize, size_t newSize);

StringObj lol_to_str(LolValue value);
StringObj *lol_alloc_lit_str(char *chars, int length);
Obj *lol_alloc_obj(size_t size, ObjType type, bool constant);
StringObj *lol_alloc_str(char *chars, int length);
StringObj lol_concat_str(size_t len, ...);
StringObj *lol_alloc_stack_str(StringObj obj);

void lol_readline(LolValue *val);
#endif
