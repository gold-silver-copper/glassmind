#!/bin/sh
# A stub loop agent for CI: makes one fixed, real improvement (a trailing
# "kthxbye" adds nothing to the meaning) so one loop round can be tested end
# to end without an LLM. Usage: stub-agent.sh WORKTREE BRIEF
set -e
cd "$1"
grep -q "round 1" "$2"
cat > crates/english/src/politeness.rs <<'EOF'
//! A sign-off that adds nothing to what is asked.

use glassmind_grammar::{Rule, RuleInfo, Sem as S, Sym};
use glassmind_meaning::Ontology;

use crate::sem::{Cat, Sem};

pub fn rules<O: Ontology>() -> Vec<Rule<Cat, Sem<O>>> {
    vec![
        // "what calls `x` kthxbye": a trailing sign-off leaves the request as it is.
        Rule::new(
            RuleInfo::new("en.utterance.stub_sign_off", 1),
            Cat::Utterance,
            vec![Sym::Cat(Cat::Utterance), Sym::Word("kthxbye")],
            S::Build(|c: &[Sem<O>]| c.first().cloned()),
        )
        .read_only(),
    ]
}
EOF
sed -i 's/^pub mod lexicon;/pub mod lexicon;\npub mod politeness;/' crates/english/src/lib.rs
sed -i 's/^        seed::rules::<O> as fn() -> Vec<Rule<Cat, Sem<O>>>,$/&\n        politeness::rules,/' crates/english/src/lib.rs
grep -q "politeness::rules," crates/english/src/lib.rs
echo "Added a read-only rule for a trailing kthxbye."
