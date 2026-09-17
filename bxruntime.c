#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

void __print_int(int64_t a) { printf("%ld\n", a); }

void __print_bool(bool b) { printf("%s\n", (b ? "true" : "false")); }

// #define print(x) \ _Generic((x), int: __print_int, bool: __print_bool,
// default: __print_int)(x)
