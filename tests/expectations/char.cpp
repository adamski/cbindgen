#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

struct Foo {
  char32_t a;
};

extern "C" {

void root(Foo a);

} // extern "C"