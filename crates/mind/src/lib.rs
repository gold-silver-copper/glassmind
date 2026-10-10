//! The dialogue layer: from a user turn to a reply, through every reading.
//!
//! For each turn it asks the language for every reading, resolves the names
//! in each reading against the world, asks the world to answer each one, and
//! drops the readings that fail, recording why. If the surviving readings
//! agree it answers; if they differ it applies the stated preference rules
//! and says what it assumed. Everything it did goes into a [`Trace`], which
//! `--explain` prints, and the turn goes into the [`Session`], which is saved
//! to disk between runs.
//!
//! It depends on no particular language or world, only on the interfaces in
//! `glassmind_meaning::interface`.

use std::collections::BTreeSet;

use glassmind_meaning::interface::{Language, Spoken, TokenSpan, Understanding, World};
use glassmind_meaning::{Assumption, Name, Ontology, Reply, Thing, Trouble, Utterance};
use glassmind_rules::{RuleInfo, SEED, Switches};
use serde::{Deserialize, Serialize};

pub mod trace;

pub use trace::{ReadingTrace, Trace};

/// Preference: when readings differ only in which item a name means, prefer
/// the item mentioned most recently in the conversation.
pub const PREFER_RECENT_MENTION: RuleInfo = RuleInfo::new("mind.prefer.recent_mention", SEED);
/// Preference: otherwise, prefer the reading the grammar found first.
pub const PREFER_FIRST_READING: RuleInfo = RuleInfo::new("mind.prefer.first_reading", SEED);
/// Preference: when no reading can be answered, and a name was taken to
/// mean items of the wrong kind for what was asked as well as several of the
/// right kind, ask which of those was meant ("which `settings` field?")
/// rather than say a function called `settings` is no field.
pub const PREFER_RIGHT_KIND: RuleInfo = RuleInfo::new("mind.prefer.right_kind", 5);
/// When readings of a request to change something would change different
/// things, ask which one was meant rather than act on a guess.
pub const ASK_BEFORE_ACTING: RuleInfo = RuleInfo::new("mind.ask_before_acting", 5);
/// Preference: when no reading can be answered, ask which item is meant if
/// some reading only lacks that, rather than report why the others failed
/// ("rename the `bytes` field" is not about fields typed `bytes`).
pub const PREFER_ASKING: RuleInfo = RuleInfo::new("mind.prefer.asking", 5);
/// Asked why it took up an action, the system gives the request that asked
/// for it: the most recent turn understood as asking for an action ("why do
/// you want to change that?" after "rename `x` to `y`").
pub const REASON_IS_REQUEST: RuleInfo = RuleInfo::new("mind.reason.request", 6);

/// Every dialogue rule, for listing and switching.
pub const RULES: &[RuleInfo] = &[
    PREFER_RECENT_MENTION,
    PREFER_FIRST_READING,
    PREFER_RIGHT_KIND,
    ASK_BEFORE_ACTING,
    PREFER_ASKING,
    REASON_IS_REQUEST,
];

/// A source the conversation was given.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRef {
    pub name: String,
    pub root: String,
}

/// One finished turn, as remembered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct TurnRecord<O: Ontology> {
    pub text: String,
    /// What the turn was taken to mean, once names were resolved; `None` if
    /// it was not understood.
    pub meaning: Option<Utterance<O>>,
    pub reply: Reply<O>,
    /// The reply as spoken.
    pub spoken: String,
}

/// A conversation: typed data, saved to disk between runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Session<O: Ontology> {
    pub sources: Vec<SourceRef>,
    pub turns: Vec<TurnRecord<O>>,
}

impl<O: Ontology> Session<O> {
    pub fn new(sources: Vec<SourceRef>) -> Self {
        Session {
            sources,
            turns: Vec::new(),
        }
    }

    /// Items mentioned in the conversation, most recent first, each once.
    /// A turn the user asked to forget, and the request to forget it,
    /// mention nothing.
    pub fn mentions(&self) -> Vec<O::Item> {
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        let mut forgotten = 0;
        for turn in self.turns.iter().rev() {
            if turn.meaning == Some(Utterance::Forget) {
                forgotten += 1;
                continue;
            }
            if forgotten > 0 {
                forgotten -= 1;
                continue;
            }
            let in_meaning = turn.meaning.iter().flat_map(|m| {
                m.things().into_iter().filter_map(|t| match t {
                    Thing::Item(i) => Some(i.clone()),
                    _ => None,
                })
            });
            let in_reply = turn.reply.items().into_iter().cloned();
            for item in in_meaning.chain(in_reply) {
                if seen.insert(item.clone()) {
                    out.push(item);
                }
            }
        }
        out
    }
}

/// How far a turn got, for scoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Progress {
    Words,
    Sentence,
    Reference,
    Action,
    Done,
}

/// The result of one turn.
#[derive(Debug, Clone)]
pub struct Turn<O: Ontology> {
    pub reply: Reply<O>,
    pub spoken: String,
    pub progress: Progress,
    /// Tokens understood before stopping, and the token count.
    pub understood: usize,
    pub tokens: usize,
    pub trace: Trace<O>,
}

impl<O: Ontology> Turn<O> {
    /// Every rule that took part, deduplicated, in first-use order.
    pub fn rules(&self) -> Vec<RuleInfo> {
        self.trace.rules()
    }
}

/// A language and a world, in conversation.
pub struct Mind<'a, O: Ontology> {
    pub language: &'a dyn Language<O>,
    pub world: &'a dyn World<O>,
    pub switches: &'a Switches,
}

/// A reading with its names resolved: the meaning, which item each name
/// was taken to mean, and the world rules used to look them up.
type Resolved<O> = (
    Utterance<O>,
    Vec<(Name, <O as Ontology>::Item)>,
    Vec<RuleInfo>,
);

/// A reading after resolution and answering.
struct Candidate<O: Ontology> {
    reading: usize,
    meaning: Utterance<O>,
    means: Vec<(Name, O::Item)>,
    result: Result<Reply<O>, Trouble<O>>,
}

impl<O: Ontology> Mind<'_, O> {
    /// Responds to one user turn and records it in the session.
    pub fn respond(&self, session: &mut Session<O>, text: &str) -> Turn<O> {
        let tokens = self.language.tokens(text);
        let mut trace = Trace::new(text, &tokens);
        let (reply, meaning) = self.think(session, text, &mut trace);
        let progress = progress_of(&reply);
        let understood = match &reply {
            Reply::Trouble(Trouble::UnknownWord { at, .. } | Trouble::UnknownShape { at }) => *at,
            _ => tokens.len(),
        };
        let spoken = self.speak(&reply, &tokens, text, &mut trace);
        session.turns.push(TurnRecord {
            text: text.to_owned(),
            meaning,
            reply: reply.clone(),
            spoken: spoken.clone(),
        });
        Turn {
            reply,
            spoken,
            progress,
            understood,
            tokens: tokens.len(),
            trace,
        }
    }

    fn think(
        &self,
        session: &Session<O>,
        text: &str,
        trace: &mut Trace<O>,
    ) -> (Reply<O>, Option<Utterance<O>>) {
        let readings = match self.language.understand(text) {
            Understanding::Failed { trouble } => {
                trace.failure = Some(trouble.clone());
                return (Reply::Trouble(trouble), None);
            }
            Understanding::Readings(readings) => readings,
        };
        let mut candidates = Vec::new();
        for (i, reading) in readings.iter().enumerate() {
            let mut rt = ReadingTrace {
                parse: reading.parse.clone(),
                rules: reading.rules.clone(),
                meaning: reading.meaning.clone(),
                resolutions: Vec::new(),
                world_rules: Vec::new(),
                outcomes: Vec::new(),
            };
            for (meaning, means, fired) in self.resolve(&reading.meaning, &mut rt) {
                let mut world_rules = fired;
                let result = match &meaning {
                    Utterance::Ask(q) => self.world.answer(q, &mut world_rules),
                    Utterance::Command(a) => self.world.propose(a, &mut world_rules),
                    Utterance::AskReason => reason(session, self.switches, &mut world_rules),
                    Utterance::Forget => Ok(Reply::Forgotten),
                    Utterance::Tell(f) => self.world.note(f, &mut world_rules),
                };
                rt.world_rules.extend(world_rules);
                rt.outcomes.push((meaning.clone(), result.clone()));
                candidates.push(Candidate {
                    reading: i,
                    meaning,
                    means,
                    result,
                });
            }
            trace.readings.push(rt);
        }
        self.choose(session, candidates, trace)
    }

    /// Every way of resolving the reading's names to items. A name with no
    /// item yields one unresolved candidate, which the world rejects.
    fn resolve(&self, meaning: &Utterance<O>, rt: &mut ReadingTrace<O>) -> Vec<Resolved<O>> {
        let mut options: Vec<Vec<Thing<O>>> = Vec::new();
        let mut names: Vec<Option<Name>> = Vec::new();
        let mut fired = Vec::new();
        for thing in meaning.things() {
            match thing {
                Thing::Named(name) => {
                    let items = self.world.lookup(name, &mut fired);
                    rt.resolutions.push((name.clone(), items.clone()));
                    names.push(Some(name.clone()));
                    if items.is_empty() {
                        options.push(vec![thing.clone()]);
                    } else {
                        options.push(items.into_iter().map(Thing::Item).collect());
                    }
                }
                _ => {
                    names.push(None);
                    options.push(vec![thing.clone()]);
                }
            }
        }
        let mut out = Vec::new();
        for combo in cartesian(&options) {
            let mut k = 0;
            let resolved = meaning.map_things(&mut |t| {
                let next = combo.get(k).cloned().unwrap_or_else(|| t.clone());
                k += 1;
                next
            });
            let means = names
                .iter()
                .zip(&combo)
                .filter_map(|(n, t)| match (n, t) {
                    (Some(n), Thing::Item(i)) => Some((n.clone(), i.clone())),
                    _ => None,
                })
                .collect();
            out.push((resolved, means, fired.clone()));
        }
        out
    }

    /// Picks the reply from the candidates, by the preference rules.
    fn choose(
        &self,
        session: &Session<O>,
        candidates: Vec<Candidate<O>>,
        trace: &mut Trace<O>,
    ) -> (Reply<O>, Option<Utterance<O>>) {
        let (good, bad): (Vec<_>, Vec<_>) = candidates.into_iter().partition(|c| c.result.is_ok());
        let Some(first) = good.first() else {
            // Nothing survived: report the trouble of the reading that got
            // furthest, or ask back.
            let wrong_kind = |c: &Candidate<O>| matches!(c.result, Err(Trouble::WrongKind { .. }));
            let mut bad = bad;
            if self.switches.on(PREFER_RIGHT_KIND.id)
                && bad.iter().any(wrong_kind)
                && bad
                    .iter()
                    .any(|c| matches!(c.result, Err(Trouble::Ambiguous { .. })))
            {
                trace.preferences.push(PREFER_RIGHT_KIND);
                bad.retain(|c| !wrong_kind(c));
            }
            let asking = self.switches.on(PREFER_ASKING.id);
            let worst = bad
                .into_iter()
                .filter_map(|c| c.result.err().map(|t| (c.reading, c.meaning, t)))
                .max_by_key(|(_, _, t)| {
                    let asks = matches!(t, Trouble::Ambiguous { .. });
                    (asks && asking, trouble_progress(t))
                });
            return match worst {
                Some((reading, meaning, trouble)) => {
                    if asking && matches!(trouble, Trouble::Ambiguous { .. }) {
                        trace.preferences.push(PREFER_ASKING);
                    }
                    trace.chosen = Some(reading);
                    (Reply::Trouble(trouble), Some(meaning))
                }
                None => (Reply::Trouble(Trouble::Unsupported), None),
            };
        };
        let replies: BTreeSet<&Reply<O>> =
            good.iter().filter_map(|c| c.result.as_ref().ok()).collect();
        if replies.len() == 1 {
            trace.choice = format!(
                "{} surviving reading(s), all with the same answer",
                good.len()
            );
            trace.chosen = Some(first.reading);
            let reply = first
                .result
                .clone()
                .unwrap_or(Reply::Trouble(Trouble::Unsupported));
            return (reply, Some(first.meaning.clone()));
        }
        // Readings may propose the same edit in different words (with or
        // without the kind said), so only different items make it a guess.
        let edited: BTreeSet<&O::Item> = replies.iter().flat_map(|r| r.items()).collect();
        if self.switches.on(ASK_BEFORE_ACTING.id)
            && edited.len() > 1
            && good
                .iter()
                .any(|c| matches!(c.meaning, Utterance::Command(_)))
        {
            trace.preferences.push(ASK_BEFORE_ACTING);
            let candidates = edited.into_iter().cloned().collect();
            return (Reply::Trouble(Trouble::Ambiguous { candidates }), None);
        }
        // The readings disagree: prefer recent mentions, then the first reading.
        let mentions = session.mentions();
        let recency = |c: &Candidate<O>| -> usize {
            c.means
                .iter()
                .map(|(_, item)| {
                    mentions
                        .iter()
                        .position(|m| m == item)
                        .unwrap_or(usize::MAX)
                })
                .min()
                .unwrap_or(usize::MAX)
        };
        let mut pick = first;
        let mut rule = PREFER_FIRST_READING;
        if self.switches.on(PREFER_RECENT_MENTION.id)
            && let Some(best) = good.iter().min_by_key(|c| (recency(c), c.reading))
            && recency(best) != usize::MAX
        {
            pick = best;
            rule = PREFER_RECENT_MENTION;
        } else if !self.switches.on(PREFER_FIRST_READING.id) {
            return (
                Reply::Trouble(Trouble::Ambiguous {
                    candidates: good
                        .iter()
                        .flat_map(|c| c.means.iter().map(|(_, i)| i.clone()))
                        .collect(),
                }),
                None,
            );
        }
        trace.preferences.push(rule);
        trace.chosen = Some(pick.reading);
        trace.choice = format!(
            "{} surviving readings with {} different answers; chose reading {} by {}",
            good.len(),
            replies.len(),
            pick.reading + 1,
            rule.id
        );
        // Say what was assumed: the meaning of each name that differs between
        // readings, or else the reading itself.
        let mut assumptions: Vec<Assumption<O>> = pick
            .means
            .iter()
            .filter(|(name, item)| {
                good.iter()
                    .any(|c| c.means.iter().any(|(n, i)| n == name && i != item))
            })
            .map(|(name, item)| Assumption::Means {
                name: name.clone(),
                item: item.clone(),
            })
            .collect();
        if assumptions.is_empty() {
            assumptions.push(Assumption::Reading {
                meaning: pick.meaning.clone(),
            });
        }
        let inner = pick
            .result
            .clone()
            .unwrap_or(Reply::Trouble(Trouble::Unsupported));
        (
            Reply::Assuming {
                assumptions,
                reply: Box::new(inner),
            },
            Some(pick.meaning.clone()),
        )
    }

    fn speak(
        &self,
        reply: &Reply<O>,
        tokens: &[TokenSpan],
        text: &str,
        trace: &mut Trace<O>,
    ) -> String {
        let spoken = self.language.say(reply).unwrap_or_else(|| Spoken {
            text: format!("(I have no words for this reply: {reply:?})"),
            rules: Vec::new(),
        });
        trace.reply = Some(reply.clone());
        trace.reply_rules = spoken.rules.clone();
        match reply {
            Reply::Trouble(Trouble::UnknownWord { at, .. } | Trouble::UnknownShape { at }) => {
                format!("{}\n  {}", spoken.text, mark(text, tokens, *at))
            }
            _ => spoken.text,
        }
    }
}

/// Why the most recent action asked for was taken up: the turn that asked.
/// Pushes the rule it used onto `fired`.
fn reason<O: Ontology>(
    session: &Session<O>,
    switches: &Switches,
    fired: &mut Vec<RuleInfo>,
) -> Result<Reply<O>, Trouble<O>> {
    if !switches.on(REASON_IS_REQUEST.id) {
        return Err(Trouble::Unsupported);
    }
    fired.push(REASON_IS_REQUEST);
    session
        .turns
        .iter()
        .enumerate()
        .rev()
        .find_map(|(turn, t)| match &t.meaning {
            Some(Utterance::Command(action)) => Some(Reply::Reason {
                turn,
                action: action.clone(),
            }),
            _ => None,
        })
        .ok_or(Trouble::Unsupported)
}

/// The input with `▮` before the token at `at` (or at the end).
pub fn mark(text: &str, tokens: &[TokenSpan], at: usize) -> String {
    let byte = tokens.get(at).map_or(text.trim_end().len(), |t| t.start);
    let (before, after) = text.split_at(byte.min(text.len()));
    format!("{before}▮{after}")
}

fn cartesian<T: Clone>(options: &[Vec<T>]) -> Vec<Vec<T>> {
    let mut out: Vec<Vec<T>> = vec![Vec::new()];
    for choices in options {
        let mut next = Vec::new();
        for prefix in &out {
            for choice in choices {
                let mut p = prefix.clone();
                p.push(choice.clone());
                next.push(p);
            }
        }
        out = next;
        // A guard against inputs with many ambiguous names.
        out.truncate(256);
    }
    out
}

fn trouble_progress<O: Ontology>(t: &Trouble<O>) -> Progress {
    match t {
        Trouble::UnknownWord { .. } => Progress::Words,
        Trouble::UnknownShape { .. } | Trouble::TooAmbiguous => Progress::Sentence,
        Trouble::NoSuchName { .. } | Trouble::Ambiguous { .. } => Progress::Reference,
        Trouble::WrongKind { .. } | Trouble::Unsupported => Progress::Action,
    }
}

fn progress_of<O: Ontology>(reply: &Reply<O>) -> Progress {
    match reply {
        Reply::Trouble(t) => trouble_progress(t),
        Reply::Assuming { reply, .. } => progress_of(reply),
        Reply::Extent(_)
        | Reply::Location(_)
        | Reply::Account { .. }
        | Reply::Count { .. }
        | Reply::Proposal { .. }
        | Reply::Reason { .. }
        | Reply::Forgotten
        | Reply::Noted(_) => Progress::Done,
    }
}
