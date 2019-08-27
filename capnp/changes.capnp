@0x8ad2e1906b690ac

using Cxx = import "/capnp/c++.capnp";
using Json = import "/capnp/compat/json.capnp";


enum ChangeStatus {
  new @0 $Json.name("NEW");
  merged @1 $Json.name("MERGED");
  abandoned @2 $Json.name("ABANDONED");
  draft @3 $Json.name("DRAFT");
}

struct ChangeTest {
  status @0 :ChangeStatus;
}
