//! The oracle: exact expected answers for generated tasks about Rust
//! sources, computed with `syn` and `proc-macro2` from the definitions in
//! `docs/tasks.md`, independently of glassmind's own code.
//!
//! Readings of the spec where it leaves room:
//! - Files that lex but do not parse are skipped for items, relations and
//!   renames, but their identifiers still count as taken for `fresh` and
//!   `no_such_name` (conservative).
//! - "Identifier tokens in the source" are the identifiers actually written
//!   in the text: the `doc` identifier proc-macro2 synthesizes for doc
//!   comments is not one.
//! - Field-type mentions exclude lifetime names (`'a`) and keywords are
//!   never generated as `type` arguments.
//! - Calls inside nested non-function items (a `const` in a function body)
//!   belong to no function; calls in a signature are not in the body.
//! - Rename exactness: tokens inside macro invocations are never one of the
//!   allowed syntactic roles, so any such token rejects the rename. For
//!   `rename_function`, a "name in a `use` tree" is a `use` leaf name or
//!   either side of `as`; an expression path that is a shorthand field
//!   (`S { old }`) rejects the rename.
//! - `fn` declarations in `extern` blocks are functions too.

pub mod answer;
pub mod source;
pub mod tasks;
