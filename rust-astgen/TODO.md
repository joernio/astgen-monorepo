In no any particular order, stuff that should be improved as time permits.

## JSON keys should be made consts

So that we don't hardcode them in multiple places and later forget to update some of them.
For instance in,

- integration tests
- during codegen

## Parse if cargo didn't work

Currently, if we can't load via cargo, the entire process fails.
We don't have to: we can at least emit an AST with missing types/method fullnames.

## Scala codegen: handle labels

Labels for sub-rules are not lowered. We handle this manually in Joern with
extension methods, but it would be nice to find a way to do it automatically.
