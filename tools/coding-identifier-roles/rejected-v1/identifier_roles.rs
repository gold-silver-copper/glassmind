//! Exact identifier constituents where the syntax explicitly asks for a code
//! name. These add readings; ordinary English alternatives remain available.

use glassmind_english::{Cat, Sem, lexicon::IDENTIFIER};
use glassmind_grammar::{Rule, Sem as Build, Sym};
use glassmind_meaning::{Action, Thing, Utterance};
use glassmind_rules::{HAND, RuleInfo};
use glassmind_world_rust::Rust;

type Value = Sem<Rust>;

fn kind_name(children: &[Value]) -> Option<Value> {
    let (kind, name) = match children {
        [Sem::Kind { kind, .. }, Sem::Name(name)] | [Sem::Name(name), Sem::Kind { kind, .. }] => {
            (kind, name)
        }
        _ => return None,
    };
    Some(Sem::Mention {
        thing: Thing::Named(name.clone()),
        kind: Some(*kind),
    })
}

fn rename(children: &[Value]) -> Option<Value> {
    let [_, Sem::Mention { thing, kind }, _, Sem::Name(name)] = children else {
        return None;
    };
    Some(Sem::Utterance(Utterance::Command(Action::Rename {
        thing: thing.clone(),
        kind: *kind,
        name: Some(name.clone()),
    })))
}

pub fn rules() -> Vec<Rule<Cat, Value>> {
    use Cat::*;
    use Sym::{Cat as C, Class};
    [
        (
            "en.rust.identifier.kind_name",
            Mention,
            vec![C(Noun), Class(IDENTIFIER)],
            kind_name as fn(&[Value]) -> Option<Value>,
        ),
        (
            "en.rust.identifier.name_kind",
            Mention,
            vec![Class(IDENTIFIER), C(Noun)],
            kind_name,
        ),
        (
            "en.rust.identifier.destination",
            Command,
            vec![C(Rename), C(Mention), C(Into), Class(IDENTIFIER)],
            rename,
        ),
    ]
    .into_iter()
    .map(|(id, lhs, rhs, build)| {
        Rule::new(RuleInfo::new(id, HAND), lhs, rhs, Build::Build(build)).read_only()
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use glassmind_english::English;
    use glassmind_meaning::interface::{Language, Understanding};
    use glassmind_rules::Switches;

    fn meanings(english: &English<Rust>, text: &str) -> Vec<Utterance<Rust>> {
        match english.understand(text) {
            Understanding::Readings(readings) => readings
                .into_iter()
                .filter(|r| r.edits == 0 && r.discarded.is_empty())
                .map(|r| r.meaning)
                .collect(),
            Understanding::Failed { .. } => Vec::new(),
        }
    }

    #[test]
    fn explicit_names_retain_spelling_and_existing_alternatives() {
        let english = English::new(crate::rules(), &Switches::all_on());
        let original = English::new(
            crate::rules()
                .into_iter()
                .filter(|r| !r.info.id.starts_with("en.rust.identifier."))
                .collect(),
            &Switches::all_on(),
        );
        for text in [
            "where is function called?",
            "where is function caller?",
            "where is field renamed of struct Change?",
            "rename function it to saved",
            "rename function drain_events to renamed",
            "rename function `café` to naïve",
            "which functions call forget?",
        ] {
            let added = meanings(&english, text);
            assert!(!added.is_empty(), "{text}");
            for old in meanings(&original, text) {
                assert!(added.contains(&old), "lost {old:?} for {text}");
            }
        }
        let text = "rename function `café` to naïve";
        assert!(meanings(&english, text).iter().any(|u| matches!(u,
            Utterance::Command(Action::Rename { thing: Thing::Named(n), name: Some(to), .. })
                if n.0 == "café" && to.0 == "naïve")));
    }

    #[test]
    fn identifier_atom_is_single_exact_token() {
        use glassmind_grammar::Lexicon;
        let english: English<Rust> = English::new(crate::rules(), &Switches::all_on());
        let lexicon = glassmind_english::EnglishLexicon::<Rust> {
            known: Default::default(),
            standing: Default::default(),
            spellings: Default::default(),
            typos: false,
            ontology: Default::default(),
        };
        for text in ["called", "drain_events", "café", "\"renamed\""] {
            let tokens = english.tokens_of(text);
            assert_eq!(tokens.len(), 1);
            for token in &tokens {
                assert_eq!(lexicon.read_class(IDENTIFIER, token).len(), 1);
            }
        }
        for text in ["`two names`", "`a-b`", "`1name`", "`a::b`"] {
            for token in english.tokens_of(text) {
                assert!(lexicon.read_class(IDENTIFIER, &token).is_empty());
            }
        }
    }
}
