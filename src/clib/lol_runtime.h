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
typedef LolValue (*LolFn)(unsigned short args, LolValue *values);

// NAN-boxing is sort of crazy, I know
// AKA, 1 1111 1111 1111 00000000000000000000000000000000000000000000000000
#define NAN_MASK 0x7ffc000000000000
// AKA, 1 1111 1111 1111 10000000000000000000000000000000000000000000000010
#define BOOLEAN_MASK 0x7ffe000000000002
// AKA, 1 1111 1111 1111 00 000000000000000000000000000000000000000000000000
#define INTEGER_MASK 0x7ffc000000000000

#define TRUE_BITS (BOOLEAN_MASK | 3)
#define FALSE_BITS (BOOLEAN_MASK | 2)
#define NULL_BITS 0x7ffe000000000000
#define NULL_VALUE ((LolValue){.as_u64 = ((uint64_t)(NULL_BITS))})

#define IS_DOUBLE(v) (((v).as_u64 & NAN_MASK) != NAN_MASK)
#define AS_DOUBLE(v) ((v).as_f64)
#define IS_NULL(v) ((v).as_u64 == NULL_BITS)
#define IS_BOOL(v) (((v).as_u64 & BOOLEAN_MASK) == BOOLEAN_MASK)
#define IS_TRUE(v) ((v).as_u64 == TRUE_BITS)
#define IS_FALSE(v) ((v).as_u64 == FALSE_BITS)
#define IS_INT(v) (((v).as_u64 & NAN_MASK) == INTEGER_MASK)
#define IS_FUN(v) (((v).as_u64 & NAN_MASK) == FUN_MASK)

#define AS_BOOL(v) ((bool)((v).as_u64 & 0x1))
#define AS_INT(v) ((int32_t)((v).as_u64))

#define FUN_MASK 0xfffc000000000000
#define AS_FUN(v) ((LolFn)((v).as_u64 & 0xFFFFFFFFFFFF))

#define INT_VALUE(i) ((LolValue){.as_u64 = ((uint64_t)(i) | INTEGER_MASK)})
#define FUN_VALUE(f) ((LolValue){.as_u64 = ((uint64_t)(f) | FUN_MASK)}):
#define DOUBLE_VALUE(d) ((LolValue){.as_f64 = (d)})
#define BOOL_VALUE(b) ((LolValue){.as_u64 = ((b) ? TRUE_BITS : FALSE_BITS)})

typedef struct {
  uint32_t s;
  uint32_t e;
} LolSpan;

extern LolValue lol_it;

LolValue lol_call(unsigned short args, LolValue fn, LolValue *values,
                  LolSpan sp);

void lol_print(LolValue value);
void lol_println(LolValue value);
bool lol_to_bool(LolValue value);
#endif
