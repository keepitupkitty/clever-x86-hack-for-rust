#include <stddef.h>

extern int my_test_c(const char* s, ...);

int main() {
  long double f = 14.88;
  long double g = 67.69;
  size_t s = 69;

  my_test_c("The string", f, s, g);

  return 0;
}
