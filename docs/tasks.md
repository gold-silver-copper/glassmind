# Tasks and their exact answers

Every generated conversation turn is labelled with the exact `Response`
(`crates/answer`) the system should give. The oracle (`tools/oracle`)
computes these labels with `syn` and `proc-macro2`, independently of
glassmind's own code. glassmind's Rust world follows the same definitions.
This file is the contract between them.

## Sources and files

A *source* is a repository root with a short name (`tds`, `ck`, ...). Its
files are all `*.rs` files under the root, skipping any directory named
`target` and any file or directory whose name starts with `.`. Paths are
relative to the root with `/` separators. A file `syn` cannot parse is skipped.
Code inside macro invocations (`foo!(...)`, `macro_rules!`) is not seen,
except where a rule below says identifiers are counted as raw tokens.

## Items

| kind | what counts |
|---|---|
| `function` | every `fn`: module-level, nested inside other functions or blocks, in inline `mod`s, associated functions and methods in `impl` blocks, and trait functions with or without a default body |
| `struct` | every `struct` |
| `enum` | every `enum` |
| `trait` | every `trait` |
| `field` | every *named* field of a `struct` (not tuple fields, not enum variant fields) |

An item is keyed by `source`, `file`, the 1-based `line` of its name
identifier, `name`, and `kind`.

A name is *unique* when exactly one item of any kind has it.

## Relations

- **calls.** Function `F` calls the name `N` when `F`'s own body contains a
  call `path(...)` whose path's last segment is `N`, or a method call
  `x.N(...)`. `F`'s own body includes closures and `async` blocks inside it,
  but not the bodies of items (functions, impls) nested inside it, which
  belong to those items. Trait default bodies count.
- **has field.** A struct has the fields declared in its body.
- **field type mentions T.** A field's type mentions `T` when `T` is one of
  the identifiers in the field's type tokens (`Vec<Velocity>` and
  `Option<physics::Velocity>` both mention `Velocity`).

## Task kinds

| kind | arguments | expected response |
|---|---|---|
| `callers` | `name`, unique, a function | `items`: every function that calls `name` (itself too, if recursive) |
| `callees` | `name`, unique, a function | `items`: every function `G` whose name is unique and which `name` calls |
| `count_callers` | `name`, unique, a function | `count`: the number of `callers` |
| `locate` | `name`, unique | `items`: that one item |
| `describe` | `name`, unique, a function, struct, enum or trait | `describe`: that item |
| `fields_of` | `name`, unique, a struct | `items`: its fields |
| `structs_with_field_type` | `type` | `items`: structs with a field whose type mentions `type`; at least one |
| `structs_with_field_named` | `field` | `items`: structs with a field named `field`; at least one |
| `rename_field` | `struct`, `old`, `new` | `edit`: see below |
| `rename_function` | `old`, `new` | `edit`: see below |
| `no_such_name` | `name` (and the kind of question asked about it) | `refuse` with `no_such_name` and that name |
| `fields_of_function` | `name`, unique, a function | `refuse` with `wrong_kind` |
| `ambiguous_rename_field` | `old`, `new`, no struct given | `ask`: every field named `old` |

### Renames

A rename task is only generated when renaming is exact without type
information:

- `rename_field`: `old` is the name of a field of exactly one struct, and
  every identifier token `old` in the source (lexed with `proc-macro2`,
  including inside macro invocations) is the field's declaration, a field
  access `expr.old`, a field in a struct expression `S { old: v }` or a field
  in a struct pattern `S { old: p }`. No shorthand `S { old }` occurs.
- `rename_function`: `old` is a unique name, a function, and every
  identifier token `old` in the source is its declaration, the last segment
  of a call path or other expression path, a method call's method name, or a
  name in a `use` tree.
- `new` is not an identifier anywhere in the source.

The expected `edit` has one `change` per identifier token `old`: its file,
1-based line, 0-based character column, `old` and `new`.

`ambiguous_rename_field` uses a field name declared by two or more structs;
the user does not say which struct.

## Conversation turns

Turns can also refer back to earlier turns. Their labels follow from the
conversation plan:

| turn | expected response |
|---|---|
| a request using `it`, `that`, `the struct`, ... for an item from an earlier turn | the same as the task with the item named |
| a correction (`no, I meant X`) | the previous task with `X` in place of the old name |
| a stated fact (`` `Player` is the hero ``) | `meta` `noted`; later turns may use the alias |
| show the context | `meta` `show_context` |
| forget that | `meta` `forget` |
| why did you change that | `meta` `explain_action` with the 0-based index of the turn that proposed the edit |
