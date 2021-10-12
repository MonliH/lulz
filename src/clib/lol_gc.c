#include "lol_gc.h"
#include "lol_runtime.h"
#include <stdio.h>

static void mark_roots() {}

void lol_add_local(LolValue v) {
  lol_vec_append(lol_stack, v);
}

void lol_pop_n_locals(size_t n) {
  lol_stack->len -= n;
}

void lol_init_stack() {
  lol_stack = lol_alloc_stack_vec(lol_init_vec());
}

void collect_garbage() {
#ifdef LOL_DEBUG_CHECK
  printf("-- gc begin\n");
#endif

  mark_roots();

#ifdef DEBUG_LOG_GC
  printf("-- gc end\n");
#endif
}
