#include "lol_runtime.h"
#include <stdint.h>
#include <inttypes.h>
#include <stdio.h>

LolValue lol_it = NULL_VALUE;

LolValue lol_call(unsigned short args, LolValue fn, LolValue *values,
                  LolSpan sp) {
  if (!IS_FUN(fn)) {
    exit(1);
  }

  LolFn func = AS_FUN(fn);

  return func(args, values);
}

void lol_print(LolValue value) {
  if (IS_INT(value)) {
    printf("%" PRId32 "", AS_INT(value));
  } else if (IS_DOUBLE(value)) {
    printf("%g", AS_DOUBLE(value));
  } else if (IS_BOOL(value)) {
    printf("%s", AS_BOOL(value) ? "WIN" : "FAIL");
  } else if (IS_NULL(value)) {
    printf("NOOB");
  } else if (IS_FUN(value)) {
    printf("<FUNKSHON at 0x%08lx>", (uint64_t)AS_FUN(value));
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
  } else if (IS_FUN(value)) {
    return true;
  } else {
    printf("internal error lol_to_bool\n");
    exit(1);
  }
}

void lol_println(LolValue value) {
  lol_print(value);
  printf("\n");
}
