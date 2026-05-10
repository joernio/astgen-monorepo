In no any particular order, stuff that should be improved as time permits.

## JSON keys should be made consts

So that we don't hardcode them in multiple places and later forget to update some of them.
For instance in,

- integration tests
- during codegen

## Parse if cargo didn't work

Currently, if we can't load via cargo, the entire process fails.
We don't have to: we can at least emit an AST with missing types/method fullnames.

## Expand macros

So that the tree doesn't end up with TOKEN_STREAM nodes, but instead with proper sub-ASTs.

## Don't pretty print JSON

No need, unless we ask it to. Shouldn't be the default.

## Scala codegen: handle labels

Labels for sub-rules are not lowered. We handle this manually in Joern with
extension methods, but it would be nice to find a way to do it automatically.

## Automatically derive `token_name_to_json_kind`

It currently matches the `Debug` representation of a `SyntaxNode`, but it
would be nicer to have guaranteed consistency, so that updates to rust-analyzer
don't inadvertently break the JSON output.

We would need to map each `SyntaxKind` to a `String` and use this mapping both
in scala-gen and json-gen.