//! Words as users often mistype them, each read as the word meant wherever
//! that word is read. A `Mistyped` rule's words are the word as typed, then
//! the word meant; it is never a phrase, so the system never says one. A
//! mistyping is a known word, so it is never read as a name; quote it in
//! backticks to name it. The words for a world's kinds are mistyped in its
//! bridge crate.

use glassmind_grammar::Rule;
use glassmind_meaning::Ontology;

use crate::locate::fillers;
use crate::sem::{Cat, Sem};

/// Mistypings from round 18's questions about structs with a field of some
/// name: "whcih structs have field `area`", "which structs hav `kinds`
/// field", "what structs have teh field more_wanted", "structs contianing
/// field reason", "what structs have a field caled value".
const MISTYPED: &[(Cat, &[(&str, &str)])] = &[(
    Cat::Mistyped,
    &[
        ("en.mistyped.whcih", "whcih which"),
        ("en.mistyped.whihc", "whihc which"),
        ("en.mistyped.wich", "wich which"),
        ("en.mistyped.hav", "hav have"),
        ("en.mistyped.hve", "hve have"),
        ("en.mistyped.hvae", "hvae have"),
        ("en.mistyped.contianing", "contianing containing"),
        ("en.mistyped.containg", "containg containing"),
        ("en.mistyped.caled", "caled called"),
        ("en.mistyped.namde", "namde named"),
        ("en.mistyped.teh", "teh the"),
    ],
)];

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    fillers(18, MISTYPED)
}
