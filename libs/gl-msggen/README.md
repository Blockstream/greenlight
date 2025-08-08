# `gl-msggen` -- Generating CLN Interfaces

This is a variant of the `msggen` tool from the CLN repository, used
to generate things in the shape of the CLN interface. In our case that
is the following artefacts used in Greenlight:

 - `UDL models`: We use `uniffi` for our public language
   bindings. Since they are essentially just forwarding the CLN
   interface we can use the CLN tooling to generate those files.

Whenever possible we use the convention of using request and response
structs, passed to a function with a single argument (the request
struct) and returning a single struct (the response struct). This is
in contrast with the flattened type of interface, in which the
function arguments are exploded from the struct. The advantage is that
it prevents us from having to explode the interface ourselves, and it
matching the grpc semantics more closely.
