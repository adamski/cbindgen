#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

struct Foo {
  float x;
};

extern "C" {

void root(Foo a);

} // extern "C"
