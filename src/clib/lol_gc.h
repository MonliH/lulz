#ifndef LOL_GC_H_INCLUDED
#define LOL_GC_H_INCLUDED
#include "lol_runtime.h"

void lol_collect_garbage();
VectorObj* lol_stack;
void lol_init_stack();
void lol_add_local(LolValue local);
void lol_pop_n_locals(size_t n);
#endif
