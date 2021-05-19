#ifndef LOL_RUNTIME_H_INCLUDED
#define LOL_RUNTIME_H_INCLUDED
#include <stdbool.h>
#include <stdlib.h>

typedef enum {
  LOL_TROOF,
  LOL_NOOB,
  LOL_NUMBAR,
  LOL_NUMBR,
  LOL_FUNKSHON,
} LolValTy;

struct LolValue;
typedef struct LolValue (*LolFn)(unsigned short args, struct LolValue* values);

typedef struct Function {
  unsigned short args;
  LolFn fn;
} Function;

typedef struct LolValue {
  LolValTy type;
  union {
    bool troof;
    double numbar;
    long numbr;
    Function funkshon;
  } as;
} LolValue;

extern LolValue lol_it;

LolValue lol_call(unsigned short args, LolValue fn, LolValue* values);
#endif
