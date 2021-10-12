#ifndef LOL_RUNTIME_H_INCLUDED
#define LOL_RUNTIME_H_INCLUDED
#define LOL_DEBUG_CHECK
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
// AKA, 0 1111 1111 1111 100000000000000000011111111111111111111111111111111
#define INTEGER_OVERFLOW 0x7ffc0000ffffffff

#define TRUE_VALUE ((LolValue){.as_u64 = (BOOLEAN_MASK | 3)})
#define FALSE_VALUE ((LolValue){.as_u64 = (BOOLEAN_MASK | 2)})
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

#define INT_VALUE(i)                                                           \
  ((LolValue){.as_u64 = (((int32_t)(i) | INTEGER_MASK) & INTEGER_OVERFLOW)})
#define FUN_VALUE(f) ((LolValue){.as_u64 = ((uint64_t)(f) | FUN_MASK)})
#define OBJ_VALUE(o) ((LolValue){.as_u64 = ((uint64_t)(o) | OBJ_MASK)})
#define DOUBLE_VALUE(d) ((LolValue){.as_f64 = (d)})
#define BOOL_VALUE(b) ((b) ? TRUE_VALUE : FALSE_VALUE)

#define OBJ_TYPE(value) (OBJ_VALUE(value)->ty)

#define IS_STR(value) lol_is_obj_ty(value, OBJ_STRING)
#define AS_STR(value) ((StringObj *)AS_OBJ(value))
#define AS_CSTR(value) (((StringObj *)AS_OBJ(value))->chars)

#define IS_VEC(value) lol_is_obj_ty(value, OBJ_VECTOR)
#define AS_VEC(value) ((VectorObj *)AS_OBJ(value))

#define IS_CLOSURE(value) lol_is_obj_ty(value, OBJ_CLOSURE)
#define AS_CLOSURE(value) ((ClosureObj *)AS_OBJ(value))

#define IS_DYNPTR(value) lol_is_obj_ty(value, OBJ_PTR)
#define AS_DYNPTR(value) ((DynPtrObj *)AS_OBJ(value))

#define ALLOCATE(ty, count) (ty *)(lol_realloc(NULL, 0, sizeof(ty) * (count)))

#define ALLOCATE_OBJ(ty, object_type)                                          \
  (ty *)(lol_alloc_obj(sizeof(ty), object_type))

#define FREE(type, pointer) lol_realloc((pointer), sizeof(type), 0)

#define MAKE_STR_OBJ(str, len, constant)                                       \
  (StringObj) { (Obj){OBJ_STRING}, (len), (str), (constant) }

#define FREE_ARRAY(type, pointer, oldCount)                                    \
  lol_realloc((pointer), sizeof(type) * (oldCount), 0)

#define GROW_VEC(ty, ptr, old_len, new_len)                                    \
  (ty *)lol_realloc(ptr, sizeof(ty) * (old_len), sizeof(ty) * (new_len))

typedef enum {
  OBJ_STRING,
  OBJ_VECTOR,
  OBJ_CLOSURE,
  OBJ_PTR,
} ObjType;

typedef struct {
  ObjType ty;
} Obj;

typedef struct {
  Obj obj;
  size_t len;
  size_t cap;
  LolValue *items;
} VectorObj;

typedef struct {
  Obj obj;
  LolValue *ptr;
} DynPtrObj;

typedef struct {
  Obj obj;
  size_t len;
  char *chars;
  bool constant;
} StringObj;

struct ClosureObj;

typedef LolValue (*LolClosureFn)(uint8_t args, LolValue *values,
                                 DynPtrObj **env);

typedef struct ClosureObj {
  Obj obj;
  LolClosureFn fn;
  DynPtrObj **upvalues;
  size_t upvalue_count;
} ClosureObj;

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
Obj *lol_alloc_obj(size_t size, ObjType type);
StringObj *lol_alloc_str(char *chars, int length);
StringObj lol_concat_str(size_t len, ...);
StringObj lol_interp_str(size_t fragments, ...);
StringObj *lol_alloc_stack_str(StringObj obj);

void lol_readline(LolValue *val);

VectorObj lol_init_vec();
VectorObj *lol_alloc_stack_vec(VectorObj obj);
void lol_vec_append(VectorObj *vec, LolValue val);

LolValue lol_vec_lit(size_t cap, size_t len, ...);

ClosureObj lol_init_closure(LolClosureFn fn, size_t upvalue_size, ...);
ClosureObj *lol_alloc_stack_closure(ClosureObj obj);

DynPtrObj lol_init_dyn_ptr(LolValue *obj);
DynPtrObj *lol_alloc_stack_dyn_ptr(DynPtrObj obj);
void lol_box_dyn_ptr(DynPtrObj *ptr);

void lol_free(Obj *obj);
#endif
