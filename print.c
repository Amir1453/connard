#include <stdbool.h>
#include <stdio.h>

void __print_int(int a) { printf("%d\n", a); }

void __print_bool(bool b) { printf("%b\n", b); }

#define print(x)                                                               \
  _Generic((x), int: __print_int, bool: __print_bool, default: __print_int)(x)
