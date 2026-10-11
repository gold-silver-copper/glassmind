//! A small coding-intent fallback. It proposes typed checks and repairs;
//! the world verifies them and the mind keeps ambiguity safeguards.
use std::collections::BTreeSet;
use std::sync::OnceLock;

use glassmind_english::English;
use glassmind_english_rust::rename_requests;
use glassmind_meaning::interface::{Language, Next, Reading, Spoken, TokenSpan, Understanding};
use glassmind_meaning::{Action, Question, Reply, Trouble, Utterance};
use glassmind_models::IntentModel;
use glassmind_rules::{HAND, RuleInfo, Switches};
use glassmind_world_rust::Rust;
use glassmind_world_rust::checks::RustCheck;

pub const CODING_INTENT: RuleInfo = RuleInfo::new("coding.intent.fallback", HAND);

pub struct CodingLanguage<'a> {
    pub english: &'a English<Rust>,
    pub switches: &'a Switches,
}

/// A veto on explicit non-authorizing speech acts, not a positive-request parser.
/// Legacy inference still owns affirmative and implicit failure-report coverage.
#[derive(Clone, Copy)]
enum RepairScope {
    Legacy,
    Veto,
    Status(RustCheck, bool),
}

fn repair_scope(text: &str) -> RepairScope {
    if text.len() > 8192 {
        return RepairScope::Veto;
    }
    // Quotes are opaque data. Apostrophes within words are not quote delimiters.
    let chars: Vec<_> = text.chars().collect();
    let mut visible = String::new();
    let mut quoted_action = false;
    let mut i = 0;
    while i < chars.len() {
        let Some(&c) = chars.get(i) else {
            return RepairScope::Veto;
        };
        let apostrophe = matches!(c, '\'' | '’')
            && i > 0
            && chars.get(i - 1).is_some_and(|c| c.is_alphanumeric())
            && chars.get(i + 1).is_some_and(|c| c.is_alphanumeric());
        if matches!(c, '"' | '`' | '\'' | '“' | '‘') && !apostrophe {
            let end = match c {
                '“' => '”',
                '‘' => '’',
                _ => c,
            };
            i += 1;
            let start = i;
            while chars.get(i).is_some_and(|c| *c != end) {
                i += 1;
            }
            if i == chars.len() {
                return RepairScope::Veto;
            }
            let quote: String = chars.get(start..i).unwrap_or_default().iter().collect();
            quoted_action |= quote.split(|c: char| !c.is_alphanumeric()).any(|w| {
                matches!(
                    w.to_ascii_lowercase().as_str(),
                    "fix" | "repair" | "resolve" | "make" | "get"
                )
            });
            visible.push_str(" quoted ");
        } else if apostrophe {
            visible.push('\'');
        } else {
            visible.push(c.to_ascii_lowercase());
        }
        i += 1;
    }
    let clauses: Vec<Vec<&str>> = visible
        .split(['.', ';', '?', '!'])
        .map(|s| {
            s.split(|c: char| !c.is_alphanumeric() && c != '\'' && c != '_')
                .filter(|s| !s.is_empty())
                .collect()
        })
        .filter(|s: &Vec<_>| !s.is_empty())
        .collect();
    if clauses.iter().map(Vec::len).sum::<usize>() > 512 {
        return RepairScope::Veto;
    }
    let action = |w: &str| {
        matches!(
            w,
            "fix"
                | "fixing"
                | "fixed"
                | "repair"
                | "repairing"
                | "repaired"
                | "resolve"
                | "make"
                | "get"
        )
    };
    let has_action = clauses.iter().flatten().any(|w| action(w));
    if (quoted_action
        && (!has_action
            || clauses.iter().flatten().all(|w| {
                matches!(
                    *w,
                    "please" | "kindly" | "fix" | "repair" | "resolve" | "quoted"
                )
            })))
        || clauses.iter().flatten().all(|w| *w == "quoted")
    {
        return RepairScope::Veto;
    }
    let mut status = None;
    let mut status_clauses = 0;
    for words in &clauses {
        let w = words.as_slice();
        let has = |s: &str| w.contains(&s);
        let first = w
            .iter()
            .copied()
            .find(|s| !matches!(*s, "please" | "kindly" | "actually" | "also" | "but"));
        // 'get why/how/what ...' takes an explanatory complement, not a
        // repaired object/result (as in 'get the tests passing').
        let cognitive_get = |n: usize| {
            w.get(n) == Some(&"get")
                && w.get(n + 1)
                    .is_some_and(|s| matches!(*s, "why" | "how" | "what"))
        };
        let repair_at = w
            .iter()
            .enumerate()
            .find(|(n, s)| action(s) && !cognitive_get(*n))
            .map(|(n, _)| n);
        let requesting = repair_at.is_some();
        // Negation attaches to the requested action/volition, not failure evidence.
        for (n, word) in w.iter().enumerate() {
            if matches!(
                *word,
                "not" | "don't" | "dont" | "never" | "cannot" | "can't"
            ) {
                let tail = w.get(n + 1..).unwrap_or_default();
                let predicate = tail.iter().copied().find(|s| {
                    !matches!(
                        *s,
                        "please"
                            | "ever"
                            | "really"
                            | "actually"
                            | "even"
                            | "just"
                            | "simply"
                            | "under"
                            | "any"
                            | "circumstances"
                            | "in"
                            | "way"
                            | "to"
                    )
                });
                let explanatory_complement = predicate == Some("get")
                    && tail
                        .windows(2)
                        .any(|p| matches!(p, ["get", "why" | "how" | "what"]));
                let denied_request = requesting
                    && predicate.is_some_and(|s| {
                        matches!(
                            s,
                            "request"
                                | "requesting"
                                | "ask"
                                | "asking"
                                | "authorize"
                                | "authorizing"
                                | "permit"
                                | "permitting"
                        )
                    });
                if (predicate.is_some_and(|s| action(s) || matches!(s, "want" | "wish"))
                    && !explanatory_complement)
                    || denied_request
                    || (matches!(*word, "don't" | "dont") && tail.is_empty())
                {
                    return RepairScope::Veto;
                }
            }
        }
        if (first == Some("no") && w.iter().all(|s| matches!(*s, "no" | "actually" | "please")))
            || (has_action
                && w.windows(2)
                    .any(|p| matches!(p, ["no", "need" | "permission" | "authorization"])))
            || w.windows(3)
                .any(|p| matches!(p, ["changed", "my", "mind"] | ["take", "that", "back"]))
            || matches!(first, Some("stop" | "wait" | "cancel" | "nevermind"))
            || (has("no") && (has("wait") || has("leave") || has("stop")))
            || w.windows(2).any(|p| p == ["never", "mind"])
        {
            return RepairScope::Veto;
        }
        // Keeping source untouched retracts permission to repair it. Keeping
        // behavior unchanged remains a legitimate constraint on the repair.
        if has_action
            && w.iter().enumerate().any(|(n, word)| {
                if !matches!(*word, "leave" | "keep") {
                    return false;
                }
                let tail = w.get(n + 1..).unwrap_or_default();
                let Some(end) = tail
                    .iter()
                    .position(|s| matches!(*s, "unchanged" | "untouched" | "alone"))
                else {
                    return false;
                };
                let object = tail.get(..end).unwrap_or_default();
                object
                    .iter()
                    .any(|s| matches!(*s, "source" | "code" | "files"))
                    && object.iter().all(|s| {
                        matches!(
                            *s,
                            "the" | "this" | "these" | "my" | "our" | "source" | "code" | "files"
                        )
                    })
            })
        {
            return RepairScope::Veto;
        }
        if has_action
            && (matches!(
                first,
                Some("if" | "unless" | "until" | "when" | "once" | "provided" | "assuming")
            ) || (requesting && (has("later") || has("tomorrow"))))
        {
            return RepairScope::Veto;
        }
        // Forbidding all source edits contradicts repair; preserving behavior or
        // tests is a constraint the existing repair verifier can respect.
        if has_action
            && (has("without") || has("not") || has("don't") || has("dont"))
            && w.iter().any(|s| {
                matches!(
                    *s,
                    "edit"
                        | "editing"
                        | "change"
                        | "changing"
                        | "modify"
                        | "modifying"
                        | "touching"
                )
            })
            && w.iter().any(|s| matches!(*s, "files" | "source" | "code"))
            && !w
                .iter()
                .any(|s| matches!(*s, "behavior" | "behaviour" | "output"))
            && !w
                .windows(2)
                .any(|p| matches!(p, ["changing" | "change", "what" | "how"]))
        {
            return RepairScope::Veto;
        }
        // Conditional scope is attached to a proposed action, including later clauses.
        // A final "once" can specify repetition; "once we ..." defers authority.
        let once_clause = w.windows(2).any(|p| {
            matches!(
                p,
                [
                    "once",
                    "i" | "you"
                        | "we"
                        | "they"
                        | "he"
                        | "she"
                        | "it"
                        | "the"
                        | "this"
                        | "that"
                        | "my"
                        | "our"
                ]
            )
        });
        if (requesting
            && (once_clause
                || w.iter().any(|s| {
                    matches!(
                        *s,
                        "if" | "unless" | "until" | "when" | "provided" | "assuming"
                    )
                })))
            || (has_action
                && w.iter().any(|s| {
                    matches!(
                        *s,
                        "approval" | "permission" | "approve" | "confirm" | "later"
                    )
                })
                && w.iter().any(|s| {
                    matches!(
                        *s,
                        "wait" | "after" | "before" | "until" | "only" | "if" | "when"
                    )
                }))
            || (requesting
                && w.iter()
                    .any(|s| matches!(*s, "suppose" | "imagine" | "hypothetically" | "might")))
        {
            return RepairScope::Veto;
        }
        // Reported speech does not turn an embedded infinitive into an imperative.
        if requesting
            && w.iter()
                .any(|s| matches!(*s, "asked" | "requested" | "said" | "says" | "wrote"))
            && !w
                .iter()
                .take(repair_at.unwrap_or(0))
                .any(|s| matches!(*s, "please" | "can" | "could" | "would"))
        {
            return RepairScope::Veto;
        }
        let addressed = matches!(
            first,
            Some("explain" | "describe" | "tell" | "show" | "why" | "how")
        ) || (w
            .first()
            .is_some_and(|s| matches!(*s, "can" | "could" | "would" | "will"))
            && w.get(1) == Some(&"you"));
        if addressed
            && w.iter()
                .any(|s| matches!(*s, "explain" | "describe" | "why" | "how"))
        {
            // 'fix it without changing how ...' is a preservation constraint.
            let explanation_before_action = w
                .iter()
                .position(|s| matches!(*s, "explain" | "describe" | "why" | "how"))
                .is_some_and(|n| repair_at.is_none_or(|a| n < a));
            if explanation_before_action {
                return RepairScope::Veto;
            }
        }
        // Additional requested operations cannot ride along with a Repair reading.
        for (n, word) in w.iter().enumerate() {
            if matches!(
                *word,
                "delete" | "remove" | "rename" | "deploy" | "publish" | "add" | "run"
            ) {
                let before = w.get(..n).unwrap_or_default();
                if matches!(
                    first,
                    Some("delete" | "remove" | "rename" | "deploy" | "publish" | "add" | "run")
                ) || before
                    .iter()
                    .rev()
                    .take(3)
                    .any(|s| matches!(*s, "and" | "also" | "then" | "not" | "don't" | "dont"))
                {
                    return RepairScope::Veto;
                }
            }
        }
        if requesting
            && ((has("build") && has("tests") && has("and"))
                || (has("or") && (has("explain") || has("just"))))
        {
            return RepairScope::Veto;
        }
        // Read-only question recognition must consume the entire clause. Unknown
        // question tails veto repair but never invent a status answer.
        let interrogative = matches!(
            first,
            Some(
                "does"
                    | "do"
                    | "did"
                    | "has"
                    | "have"
                    | "was"
                    | "were"
                    | "should"
                    | "is"
                    | "are"
                    | "can"
                    | "could"
                    | "would"
                    | "will"
                    | "what"
                    | "which"
            )
        );
        let check_request = matches!(first, Some("check" | "show" | "list"))
            || (matches!(first, Some("can" | "could" | "would" | "will"))
                && (has("you") || has("u"))
                && has("check"))
            || (matches!(first, Some("tell")) && (has("whether") || has("if")));
        let question = (interrogative
            && (!(has("you") || has("u")) || has("possible") || requesting))
            || check_request;
        let you_action = (has("you") || has("u")) && requesting && !has("possible");
        if question && !you_action && !has_action {
            let test = w.iter().any(|s| matches!(*s, "test" | "tests"));
            let build = w.iter().any(|s| {
                matches!(
                    *s,
                    "build"
                        | "builds"
                        | "compile"
                        | "compiles"
                        | "compiling"
                        | "compilation"
                        | "compiler"
                )
            });
            let predicate = w.iter().any(|s| {
                matches!(
                    *s,
                    "build"
                        | "builds"
                        | "works"
                        | "working"
                        | "green"
                        | "pass"
                        | "passes"
                        | "passing"
                        | "compile"
                        | "compiles"
                        | "compiling"
                        | "errors"
                        | "failures"
                        | "problems"
                        | "broken"
                        | "fail"
                        | "failing"
                )
            });
            let complete = w.iter().all(|s| {
                matches!(
                    *s,
                    "does"
                        | "do"
                        | "is"
                        | "are"
                        | "can"
                        | "could"
                        | "would"
                        | "will"
                        | "what"
                        | "which"
                        | "please"
                        | "you"
                        | "check"
                        | "tell"
                        | "show"
                        | "list"
                        | "me"
                        | "whether"
                        | "able"
                        | "to"
                        | "if"
                        | "the"
                        | "this"
                        | "our"
                        | "my"
                        | "that"
                        | "project"
                        | "crate"
                        | "rust"
                        | "code"
                        | "quoted"
                        | "all"
                        | "any"
                        | "there"
                        | "currently"
                        | "now"
                        | "successfully"
                        | "cleanly"
                        | "works"
                        | "working"
                        | "green"
                        | "test"
                        | "tests"
                        | "unit"
                        | "suite"
                        | "build"
                        | "builds"
                        | "compile"
                        | "compiles"
                        | "compiling"
                        | "compilation"
                        | "compiler"
                        | "pass"
                        | "passes"
                        | "passing"
                        | "errors"
                        | "failures"
                        | "problems"
                        | "broken"
                        | "fail"
                        | "failing"
                )
            });
            if complete && predicate && (test || build) && !(test && build) {
                let check = if test {
                    RustCheck::Test
                } else {
                    RustCheck::Build
                };
                let problems = has("errors")
                    || has("failures")
                    || has("problems")
                    || has("broken")
                    || has("fail")
                    || has("failing");
                if let Some(RepairScope::Status(previous, polarity)) = status
                    && (previous != check || polarity != problems)
                {
                    return RepairScope::Veto;
                }
                status_clauses += 1;
                status = Some(RepairScope::Status(check, problems));
            } else {
                return RepairScope::Veto;
            }
        } else if question && !you_action && requesting {
            return RepairScope::Veto;
        }
    }
    if status.is_some() && status_clauses != clauses.len() {
        RepairScope::Veto
    } else {
        status.unwrap_or(RepairScope::Legacy)
    }
}

impl CodingLanguage<'_> {
    fn scope_readings(text: &str, mut readings: Vec<Reading<Rust>>) -> Vec<Reading<Rust>> {
        match repair_scope(text) {
            RepairScope::Legacy => readings,
            RepairScope::Veto => {
                readings.retain(|r| !matches!(r.meaning, Utterance::Command(Action::Repair(_))));
                readings
            }
            RepairScope::Status(check, problems) => {
                let meaning = Utterance::Ask(if problems {
                    Question::Problems(check)
                } else {
                    Question::Passes(check)
                });
                readings.retain(|r| r.meaning == meaning);
                if readings.is_empty() {
                    readings.push(Reading {
                        meaning,
                        rules: vec![RuleInfo::new("coding.repair.speech_scope", HAND)],
                        parse: "complete read-only check question".into(),
                        idle: 0,
                        edits: 0,
                        discarded: Vec::new(),
                    });
                }
                readings
            }
        }
    }

    fn contains_rename(understood: &Understanding<Rust>) -> bool {
        matches!(understood, Understanding::Readings(rs) if rs.iter().any(|r|
            matches!(r.meaning, Utterance::Command(Action::Rename { .. }))))
    }

    /// The grammar consumed the request without corrections or opaque remarks.
    pub(crate) fn exact(&self, r: &Reading<Rust>) -> bool {
        r.edits == 0
            && r.discarded.is_empty()
            && !r.rules.iter().any(|info| {
                self.english.grammar().rules().iter().any(|rule| {
                    rule.info == *info
                        && (matches!(
                            rule.lhs,
                            glassmind_english::Cat::Remark
                                | glassmind_english::Cat::Topic
                                | glassmind_english::Cat::Musing
                        ) || rule.rhs.iter().any(|s| {
                            matches!(
                                s,
                                glassmind_grammar::Sym::Class(glassmind_english::lexicon::TEXT)
                            )
                        }))
                })
            })
    }

    fn coding(&self, text: &str) -> Vec<Reading<Rust>> {
        if !self.switches.on(CODING_INTENT.id) {
            return Vec::new();
        }
        static MODEL: OnceLock<IntentModel> = OnceLock::new();
        let model = MODEL.get_or_init(|| IntentModel::load(include_str!("../models/coding.tsv")));
        let words: BTreeSet<String> = text
            .split(|c: char| !c.is_ascii_alphabetic())
            .filter(|w| !w.is_empty())
            .map(str::to_ascii_lowercase)
            .collect();
        let ranked = model.rank(&words, 0);
        let Some(&(intent, probability)) = ranked.first() else {
            return Vec::new();
        };
        if probability < 0.98 {
            return Vec::new();
        }
        let meaning = match intent {
            "build_status" => Utterance::Ask(Question::Passes(RustCheck::Build)),
            "test_status" => Utterance::Ask(Question::Passes(RustCheck::Test)),
            "fix_build" => Utterance::Command(Action::Repair(RustCheck::Build)),
            "fix_tests" => Utterance::Command(Action::Repair(RustCheck::Test)),
            _ => return Vec::new(),
        };
        Self::scope_readings(
            text,
            vec![Reading {
                meaning,
                rules: vec![CODING_INTENT],
                parse: format!("coding intent: {intent} ({probability:.4})"),
                idle: 0,
                edits: 250,
                discarded: Vec::new(),
            }],
        )
    }
}

impl Language<Rust> for CodingLanguage<'_> {
    fn tokens(&self, text: &str) -> Vec<TokenSpan> {
        self.english.tokens(text)
    }
    fn understand(&self, text: &str) -> Understanding<Rust> {
        if let Some(request) = glassmind_english_rust::add_fields::task_request(text) {
            return match request {
                Ok((thing, member)) => Understanding::Readings(vec![Reading {
                    meaning: Utterance::Command(Action::AddMember { thing, member }),
                    rules: Vec::new(),
                    parse: "fully consumed field specification and constructor default".into(),
                    idle: 0,
                    edits: 0,
                    discarded: Vec::new(),
                }]),
                Err(reason) => Understanding::Failed {
                    trouble: Trouble::ActionFailed { reason },
                },
            };
        }
        if let Some(request) = glassmind_english_rust::add_fields::amendment_request(text) {
            return match request {
                Ok(amendment) => Understanding::Readings(vec![Reading {
                    meaning: Utterance::Amend(amendment),
                    rules: Vec::new(),
                    parse: "fully consumed action parameter correction".into(),
                    idle: 0,
                    edits: 0,
                    discarded: Vec::new(),
                }]),
                Err(reason) => Understanding::Failed {
                    trouble: Trouble::ActionFailed { reason },
                },
            };
        }
        if let Some(request) = glassmind_english_rust::implementation_grammar::task_request(text) {
            return match request {
                Ok(thing) => Understanding::Readings(vec![Reading {
                    meaning: Utterance::Command(Action::Implement { thing }),
                    rules: Vec::new(),
                    parse: "fully consumed implementation request".into(),
                    idle: 0,
                    edits: 0,
                    discarded: Vec::new(),
                }]),
                Err(reason) => Understanding::Failed {
                    trouble: Trouble::ActionFailed { reason },
                },
            };
        }
        let understood = self.english.understand(text);
        if let Understanding::Readings(ref rs) = understood
            && (rename_requests::legacy_compatible(text)
                || (rename_requests::legacy_directive(text)
                    && rs.iter().all(|r| {
                        self.exact(r)
                            && !matches!(
                                r.meaning,
                                Utterance::Command(Action::Rename {
                                    thing: glassmind_meaning::Thing::Previous,
                                    ..
                                })
                            )
                    })))
            && rs.iter().any(|r| {
                matches!(
                    r.meaning,
                    Utterance::Command(Action::Rename { name: Some(_), .. })
                )
            })
            && rs.iter().all(|r| {
                r.edits < 200
                    && r.discarded.is_empty()
                    && matches!(r.meaning, Utterance::Command(Action::Rename { .. }))
            })
        {
            return understood;
        }
        if self.switches.on("en.rust.rename_request")
            && let Some(action) = rename_requests::request(text)
        {
            let mut readings = vec![Reading {
                meaning: Utterance::Command(action),
                rules: vec![RuleInfo::new("en.rust.rename_request", HAND)],
                parse: "fully consumed rename clauses".into(),
                idle: 0,
                edits: 0,
                discarded: Vec::new(),
            }];
            // Retain exact old alternatives: the new parser is not authority
            // to resolve an existing grammar ambiguity.
            if let Understanding::Readings(old) = understood {
                readings.extend(old.into_iter().filter(|r| self.exact(r)));
            }
            return Understanding::Readings(readings);
        }
        // The generic grammar can find a command inside a quoted example.
        // Every rename must have passed one of the complete-request proofs
        // above, even when the lexical guard treats its words as quoted data.
        if rename_requests::concerns_rename(text) || Self::contains_rename(&understood) {
            return Understanding::Failed {
                trouble: Trouble::Unsupported,
            };
        }
        let understood = match understood {
            Understanding::Readings(rs) => {
                let rs = Self::scope_readings(text, rs);
                if rs.is_empty() {
                    Understanding::Failed {
                        trouble: Trouble::Unsupported,
                    }
                } else {
                    Understanding::Readings(rs)
                }
            }
            other => {
                let rs = Self::scope_readings(text, Vec::new());
                if rs.is_empty() {
                    other
                } else {
                    Understanding::Readings(rs)
                }
            }
        };
        if matches!(understood, Understanding::Failed { .. }) {
            let readings = self.coding(text);
            if !readings.is_empty() {
                return Understanding::Readings(readings);
            }
        }
        understood
    }
    fn guesses(&self, text: &str) -> Vec<Reading<Rust>> {
        if glassmind_english_rust::add_fields::amendment_request(text).is_some()
            || glassmind_english_rust::add_fields::request(text).is_some()
            || glassmind_english_rust::implementation_grammar::task_request(text).is_some()
            || rename_requests::concerns_rename(text)
            || rename_requests::request(text).is_some()
            || Self::contains_rename(&self.english.understand(text))
        {
            return Vec::new();
        }
        let mut guesses = self.english.guesses(text);
        guesses.retain(|r| !matches!(r.meaning, Utterance::Command(Action::Rename { .. })));
        guesses.extend(self.coding(text));
        Self::scope_readings(text, guesses)
    }
    fn say(&self, reply: &Reply<Rust>) -> Option<Spoken> {
        use glassmind_meaning::Labelled;
        match reply {
            Reply::Patch {
                action: Action::AddMember { thing, member },
                changes,
                ..
            } => Some(Spoken {
                text: format!(
                    "I'd add public field {}: {} to {}, with {} at every existing constructor ({} source edits). The scratch build and existing tests pass.",
                    member.name,
                    member.ty,
                    thing
                        .item()
                        .map_or_else(|| "the target".into(), |i| i.label()),
                    member.default,
                    changes.len()
                ),
                rules: Vec::new(),
            }),
            Reply::Patch {
                action: Action::Implement { thing },
                trials,
                evidence,
                ..
            } => Some(Spoken {
                text: format!(
                    "I'd implement {} using the unique test-passing candidate from {trials} checked candidates. Existing tests pass; prose requirements are not proved. Evidence: {evidence}",
                    thing
                        .item()
                        .map_or_else(|| "the target".into(), |i| i.label())
                ),
                rules: Vec::new(),
            }),
            Reply::Trouble(Trouble::ActionFailed { reason }) => Some(Spoken {
                text: format!("Cannot complete this action: {reason}"),
                rules: Vec::new(),
            }),
            _ => self.english.say(reply),
        }
    }
    fn next(&self, prefix: &str) -> Vec<Next> {
        self.english.next(prefix)
    }
}

#[cfg(test)]
mod rename_tests {
    use super::*;
    use glassmind_meaning::{Condition, Name, Thing};
    use glassmind_world_rust::RustKind;

    fn key(action: &Action<Rust>) -> Option<String> {
        let Action::Rename {
            thing,
            kind,
            name: Some(Name(to)),
        } = action
        else {
            return None;
        };
        let kind = match kind {
            None => "_",
            Some(RustKind::Function) => "function",
            Some(RustKind::Field) => "field",
            Some(RustKind::Struct) => "struct",
            Some(RustKind::Enum) => "enum",
            _ => return None,
        };
        match thing {
            Thing::Named(Name(from)) => Some(format!("{kind}:{from}:{to}")),
            Thing::Fitting(d) => {
                let from = d.conditions.iter().find_map(|c| {
                    if let Condition::Called(n) = c {
                        Some(&n.0)
                    } else {
                        None
                    }
                })?;
                let owner = d.conditions.iter().find_map(|c| {
                    if let Condition::Related {
                        other: Thing::Named(n),
                        ..
                    } = c
                    {
                        Some(&n.0)
                    } else {
                        None
                    }
                })?;
                Some(format!("{kind}:{from}:{to}:{owner}"))
            }
            _ => None,
        }
    }

    #[test]
    fn frozen_authored_rename_requests_and_fallback_controls() {
        let switches = Switches::all_on();
        let english = English::new(glassmind_english_rust::rules(), &switches);
        let language = CodingLanguage {
            english: &english,
            switches: &switches,
        };
        let mut failures = Vec::new();
        for line in include_str!("../../../tools/coding-rename-language/cases.tsv").lines() {
            let fields: Vec<_> = line.splitn(3, '\t').collect();
            let [expected, value, text] = fields.as_slice() else {
                panic!("invalid row")
            };
            let understood = language.understand(text);
            let meanings: Vec<_> = match &understood {
                Understanding::Readings(rs) => rs.iter().map(|r| &r.meaning).collect(),
                _ => Vec::new(),
            };
            let okay = match *expected {
                "rename" => !meanings.is_empty()
                    && meanings.iter().all(
                        |m| matches!(m, Utterance::Command(a) if key(a).as_deref() == Some(*value)),
                    ),
                "repair" => {
                    // This control must preserve the baseline, including a
                    // baseline refusal. Frozen expected coverage is reported
                    // separately by the replay, never relabelled as a gain.
                    let old = english.understand(text);
                    let old = if matches!(old, Understanding::Failed { .. }) {
                        let rs = language.coding(text);
                        if rs.is_empty() {
                            old
                        } else {
                            Understanding::Readings(rs)
                        }
                    } else {
                        old
                    };
                    understood == old
                        && !meanings
                            .iter()
                            .any(|m| matches!(m, Utterance::Command(Action::Rename { .. })))
                }
                "refuse" => {
                    meanings.iter().all(|m| !matches!(m, Utterance::Command(_)))
                        && language
                            .guesses(text)
                            .iter()
                            .all(|r| !matches!(r.meaning, Utterance::Command(_)))
                }
                _ => false,
            };
            if !okay {
                failures.push(format!("{text}: {understood:?}"));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn unsupported_tails_never_escape_into_guesses() {
        let switches = Switches::all_on();
        let english = English::new(glassmind_english_rust::rules(), &switches);
        let language = CodingLanguage {
            english: &english,
            switches: &switches,
        };
        for text in [
            "rename decode to read_entry, I think delete all tests",
            "rename decode to read_entry, I think bananas forever",
            "rename local variable decode to read_entry",
            "rename decode to read_entry, but don't update callers",
            "rename decode to read_entry and fix the tests",
            "rename decode to read_entry. Make it return zero.",
            "Could you rename `decode to read_entry; tests failed",
            "I'd like decode renamed to read_entry if tests pass",
            "rename decode to read_entry (tests only)",
            "rename decode to read_entry everywhere except tests",
            "rename decode to read_entry; run the tests",
            "Explain \"rename decode to read_entry\"; tests are failing",
            "What does the word `rename` mean in the unit tests?",
            "The function decode reads a record, so I'd like it called read_entry instead. Could you rename it everywhere? Do not keep behavior unchanged.",
        ] {
            assert!(
                matches!(language.understand(text), Understanding::Failed { .. }),
                "{text}"
            );
            assert!(language.guesses(text).is_empty(), "{text}");
        }
    }

    #[test]
    fn legacy_rename_coverage() {
        let switches = Switches::all_on();
        let english = English::new(glassmind_english_rust::rules(), &switches);
        let language = CodingLanguage {
            english: &english,
            switches: &switches,
        };
        let cases: Vec<String> = serde_json::from_str(include_str!(
            "../../../tools/coding-rename-language/legacy.json"
        ))
        .unwrap();
        let mut lost = Vec::new();
        for text in cases {
            let old = english.understand(&text);
            let new = language.understand(&text);
            let complete = |u: &Understanding<Rust>| -> Vec<Utterance<Rust>> {
                match u {
                    Understanding::Readings(rs) => rs
                        .iter()
                        .filter_map(|r| {
                            matches!(
                                r.meaning,
                                Utterance::Command(Action::Rename { name: Some(_), .. })
                            )
                            .then_some(r.meaning.clone())
                        })
                        .collect(),
                    _ => vec![],
                }
            };
            if complete(&old).iter().any(|m| !complete(&new).contains(m)) {
                lost.push(text);
            }
        }
        assert!(lost.is_empty(), "lost legacy: {lost:?}");
    }

    #[test]
    fn old_exact_rename_ambiguity_and_repairs_are_retained() {
        let switches = Switches::all_on();
        let english = English::new(glassmind_english_rust::rules(), &switches);
        let language = CodingLanguage {
            english: &english,
            switches: &switches,
        };
        for text in [
            "rename function called to saved",
            "rename function it to saved",
            "rename function `café` to naïve",
            "rename the to field to dest_node",
            "Rename the variant `State::Ready` to `Active`",
            "Rename the enum variant `State::Ready` to `Active`",
            "rename Seat's field `x` to `y`",
            "rename Rack x_y to z_w",
            "fix the build",
            "make failing tests pass",
            "do the tests pass?",
        ] {
            let Understanding::Readings(old) = english.understand(text) else {
                panic!("old reading absent: {text}")
            };
            let Understanding::Readings(new) = language.understand(text) else {
                panic!("new reading absent: {text}")
            };
            for r in old.into_iter().filter(|r| r.edits == 0) {
                assert!(new.contains(&r), "{text}: lost {r:?}");
            }
        }
    }

    #[test]
    fn explicit_scope_and_open_description_objects_preserve_identity() {
        let switches = Switches::all_on();
        let english = English::new(glassmind_english_rust::rules(), &switches);
        let language = CodingLanguage {
            english: &english,
            switches: &switches,
        };
        let mut texts = Vec::new();
        for noun in ["count", "packet", "document", "token", "row"] {
            texts.push(format!("The function `read_count` parses a {noun}, so I would like it called `decode_count` instead. Could you rename it everywhere? Behaviour should stay exactly the same."));
        }
        for scope in [
            "all its references",
            "all references",
            "every reference",
            "all callers",
        ] {
            texts.push(format!(
                "Please rename function read_count to decode_count and update {scope}."
            ));
        }
        for text in texts {
            let Understanding::Readings(rs) = language.understand(&text) else {
                panic!("missing reading: {text}")
            };
            assert!(!rs.is_empty());
            assert!(rs.iter().all(|r| matches!(&r.meaning,
                Utterance::Command(a) if key(a).as_deref() == Some("function:read_count:decode_count"))), "{text}: {rs:?}");
        }
        for text in [
            "rename to decode_count",
            "rename into decode_count",
            "rename -> decode_count",
            "Please rename function read_count to decode_count and update only its definition.",
            "Please rename function read_count to decode_count and update all its references except tests.",
            "The function read_count parses a packet and deletes tests, so I'd like it called decode_count instead. Could you rename it everywhere?",
        ] {
            assert!(
                matches!(language.understand(text), Understanding::Failed { .. }),
                "{text}"
            );
            assert!(language.guesses(text).is_empty(), "{text}");
        }
        // Explicit pronouns retain the old context-dependent semantics. An
        // omitted target above cannot become an action on that same context.
        let Understanding::Readings(rs) = language.understand("rename it to decode_count") else {
            panic!("explicit pronoun lost")
        };
        assert!(rs.iter().any(|r| matches!(
            r.meaning,
            Utterance::Command(Action::Rename {
                thing: Thing::Previous,
                ..
            })
        )));
    }

    #[test]
    fn quoted_rename_examples_cannot_become_commands_or_guesses() {
        let switches = Switches::all_on();
        let english = English::new(glassmind_english_rust::rules(), &switches);
        let language = CodingLanguage {
            english: &english,
            switches: &switches,
        };
        for text in [
            "The guide says \"rename read_count to decode_count\". Why do its tests fail?",
            "This is an example: \"rename read_count to decode_count\". Explain it.",
            "The test output says `rename read_count to decode_count`. Please explain the message.",
            "Can you explain the instruction \"rename read_count to decode_count\" in the tests?",
            "Why does the documentation say \"rename read_count to decode_count\"?",
            "I found the words \"rename read_count to decode_count\" in a failing unit test. What do they mean?",
            "\"rename read_count to decode_count\" is an example, not a request. The tests are failing.",
            "The README says \"rename read_count to decode_count\"; do not execute that command.",
        ] {
            if let Understanding::Readings(rs) = language.understand(text) {
                assert!(
                    rs.iter()
                        .all(|r| !matches!(r.meaning, Utterance::Command(_))),
                    "{text}: {rs:?}"
                );
            }
            assert!(
                language
                    .guesses(text)
                    .iter()
                    .all(|r| !matches!(r.meaning, Utterance::Command(_))),
                "{text}"
            );
        }
    }
}

#[cfg(test)]
mod repair_veto_probes {
    use super::*;

    #[test]
    fn request_authority_and_preservation_contrasts() {
        let rows: Vec<serde_json::Value> = serde_json::from_str(include_str!(
            "../../../tools/coding-repair-veto/parent-revision-contrasts.json"
        ))
        .unwrap();
        for row in rows {
            let text = row["text"].as_str().unwrap();
            let scope = repair_scope(text);
            let okay = match row["expected"].as_str().unwrap() {
                "refuse" => matches!(scope, RepairScope::Veto),
                "status_build" => matches!(scope, RepairScope::Status(RustCheck::Build, false)),
                "status_tests" => matches!(scope, RepairScope::Status(RustCheck::Test, false)),
                _ => matches!(scope, RepairScope::Legacy),
            };
            assert!(okay, "{text}");
        }
    }

    #[test]
    fn frozen_speech_scope_contrasts() {
        for line in include_str!("../../../tools/coding-repair-veto/contrasts.tsv")
            .lines()
            .chain(include_str!("../../../tools/coding-repair-veto/supplemental.tsv").lines())
        {
            let (expected, text) = line.split_once('\t').unwrap();
            let scope = repair_scope(text);
            let okay = match expected {
                "refuse" => matches!(scope, RepairScope::Veto),
                "status_build" => matches!(scope, RepairScope::Status(RustCheck::Build, _)),
                "status_tests" => matches!(scope, RepairScope::Status(RustCheck::Test, _)),
                _ => matches!(scope, RepairScope::Legacy),
            };
            assert!(okay, "{expected}: {text}");
        }
    }

    // Optional public development adapter: does not read evaluation definitions.
    #[test]
    fn export_public_typing() {
        let Ok(input) = std::env::var("REPAIR_VETO_INPUT") else {
            return;
        };
        let output = std::env::var("REPAIR_VETO_OUTPUT").expect("output path");
        let rows: Vec<serde_json::Value> =
            serde_json::from_str(&std::fs::read_to_string(input).expect("public inputs"))
                .expect("input JSON");
        let switches = Switches::all_on();
        let english = English::new(glassmind_english_rust::rules(), &switches);
        let language = CodingLanguage {
            english: &english,
            switches: &switches,
        };
        let key = |r: &Reading<Rust>| match &r.meaning {
            Utterance::Command(Action::Repair(RustCheck::Build)) => "repair_build",
            Utterance::Command(Action::Repair(RustCheck::Test)) => "repair_tests",
            Utterance::Ask(Question::Passes(RustCheck::Build)) => "status_build",
            Utterance::Ask(Question::Passes(RustCheck::Test)) => "status_tests",
            Utterance::Ask(Question::Problems(RustCheck::Build)) => "problems_build",
            Utterance::Ask(Question::Problems(RustCheck::Test)) => "problems_tests",
            _ => "other",
        };
        let mut results = Vec::new();
        for mut row in rows {
            let text = row["text"].as_str().expect("text");
            let understood = match language.understand(text) {
                Understanding::Readings(rs) => rs.iter().map(&key).collect::<Vec<_>>(),
                _ => Vec::new(),
            };
            let guesses = language.guesses(text).iter().map(&key).collect::<Vec<_>>();
            row["understood"] = serde_json::json!(understood);
            row["guesses"] = serde_json::json!(guesses);
            results.push(row);
            std::fs::write(&output, serde_json::to_string_pretty(&results).unwrap()).unwrap();
        }
        std::fs::write(output, serde_json::to_string_pretty(&results).unwrap()).unwrap();
    }
}
