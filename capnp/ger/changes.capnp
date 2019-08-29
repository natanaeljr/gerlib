@0xa4b6ee465f7531b2;

using Cxx = import "/capnp/c++.capnp";
using Json = import "/capnp/compat/json.capnp";

$Cxx.namespace("gerrit");

enum ChangeStatus {
  new @0 $Json.name("NEW");
  merged @1 $Json.name("MERGED");
  abandoned @2 $Json.name("ABANDONED");
  draft @3 $Json.name("DRAFT");
}
