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
use glassmind_meaning::{
    Assumption, Condition, Fact, Name, Ontology, Reply, Thing, Trouble, Utterance,
};
use glassmind_rules::{RuleInfo, SEED, Switches};
use serde::{Deserialize, Serialize};

pub mod trace;

pub use trace::{ReadingTrace, Trace, WorldResult};

/// Preference: when readings differ, keep those that take the fewest words
/// to mean nothing ("is there anything that calls `x`? like how many
/// functions call it" asks how many, rather than asking which and setting
/// the rest aside as a remark).
pub const PREFER_FEWEST_IDLE: RuleInfo = RuleInfo::new("mind.prefer.fewest_idle", 9);
/// Preference: when readings differ only in which item a name means, prefer
/// the item mentioned most recently in the conversation.
pub const PREFER_RECENT_MENTION: RuleInfo = RuleInfo::new("mind.prefer.recent_mention", SEED);
/// Preference: otherwise, prefer the reading the grammar found first.
pub const PREFER_FIRST_READING: RuleInfo = RuleInfo::new("mind.prefer.first_reading", SEED);
/// Preference: when no reading can be answered, ask which item is meant if
/// some reading only lacks that, rather than report why the others failed
/// ("rename the `bytes` field" is not about fields typed `bytes`).
pub const PREFER_ASKING: RuleInfo = RuleInfo::new("mind.prefer.asking", 5);
/// Asked why it took up an action, the system gives the request that asked
/// for it: the most recent turn understood as asking for an action ("why do
/// you want to change that?" after "rename `x` to `y`").
pub const REASON_IS_REQUEST: RuleInfo = RuleInfo::new("mind.reason.request", 6);
/// A name the user said earlier they would call an item by means that
/// item ("what fields does the slot have?" after "I'll call `CacheLine`
/// "the slot""), the latest such word holding.
pub const ALIAS_NOTED: RuleInfo = RuleInfo::new("mind.alias.noted", 7);
/// Asked what it holds of the conversation, the system gives the items
/// mentioned in it, most recent first, leaving out what the user asked it
/// to forget ("can you show me what you're keeping in your context?").
pub const RECALL_IS_MENTIONS: RuleInfo = RuleInfo::new("mind.recall.mentions", 7);
/// A thing referred back to ("where is it defined?", "where is that
/// function defined?" after "which functions call `x`?") is the one item
/// the latest understood turn asked about, if it asked about just one. A
/// thing that turn described by its name ("`tick` on `Health`") is not
/// the items that qualify it, so then there is none.
pub const PREVIOUS_IS_TOPIC: RuleInfo = RuleInfo::new("mind.refer.topic", 9);
/// A correction ("sorry, I meant `y`" after "where is `x` defined?") asks
/// the request of the latest remembered turn again, with the thing it says
/// in place of the one item that request was about. If that turn was not
/// understood, or was about no item or several, nothing is corrected.
pub const CORRECTION_REPLACES: RuleInfo = RuleInfo::new("mind.correct.replaces", 10);
/// Preference: when no reading can be answered, report why a reading
/// failed that had something to refer back to, over one whose "it" means
/// nothing yet ("I'm looking for `x`, where is it defined?" with no `x`:
/// there is nothing called `x`, rather than nothing for "it" to be).
pub const PREFER_REFERRING: RuleInfo = RuleInfo::new("mind.prefer.referring", 10);

/// Every dialogue rule, for listing and switching.
pub const RULES: &[RuleInfo] = &[
    PREFER_FEWEST_IDLE,
    PREFER_RECENT_MENTION,
    PREFER_FIRST_READING,
    PREFER_ASKING,
    REASON_IS_REQUEST,
    ALIAS_NOTED,
    RECALL_IS_MENTIONS,
    PREVIOUS_IS_TOPIC,
    CORRECTION_REPLACES,
    PREFER_REFERRING,
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
        let mut out = Vec::new();
        for turn in self.remembered() {
            for item in spoken_of(turn).chain(turn.reply.items()) {
                if !out.contains(item) {
                    out.push(item.clone());
                }
            }
        }
        out
    }

    /// The turns, most recent first, leaving out each turn the user asked
    /// to forget and the request to forget it.
    fn remembered(&self) -> impl Iterator<Item = &TurnRecord<O>> {
        let mut forgotten = 0;
        self.turns.iter().rev().filter(move |turn| {
            if turn.meaning == Some(Utterance::Forget) {
                forgotten += 1;
                false
            } else if forgotten > 0 {
                forgotten -= 1;
                false
            } else {
                true
            }
        })
    }

    /// The one item the latest understood turn asked about, if it asked
    /// about just one. An item only mentioned in describing another thing
    /// ("which functions call `tick` on `BodyHealth`") is not asked about.
    pub fn topic(&self) -> Option<O::Item> {
        let meaning = self.remembered().find_map(|t| t.meaning.as_ref())?;
        let called = |c: &Condition<O>| matches!(c, Condition::Called(_));
        if meaning
            .subjects()
            .iter()
            .any(|t| matches!(t, Thing::Fitting(d) if d.conditions.iter().any(called)))
        {
            return None;
        }
        let mut items = meaning.things().into_iter().filter_map(Thing::item);
        let item = items.next()?;
        items.next().is_none().then(|| item.clone())
    }

    /// The latest remembered turn's request, about `thing` in place of the
    /// one item it was about.
    pub fn corrected(&self, thing: &Thing<O>) -> Option<Utterance<O>> {
        let meaning = self.remembered().next()?.meaning.as_ref()?;
        let things = meaning.things();
        let [old @ Thing::Item(_)] = things.as_slice() else {
            return None;
        };
        Some(meaning.map_things(&mut |t| if t == *old { thing } else { t }.clone()))
    }

    /// The item the user last said they would call `name`, if any.
    pub fn aliased(&self, name: &Name) -> Option<O::Item> {
        self.turns.iter().rev().find_map(|turn| match &turn.reply {
            Reply::Noted(Fact::Alias {
                thing: Thing::Item(item),
                alias,
                ..
            }) if alias.0.eq_ignore_ascii_case(&name.0) => Some(item.clone()),
            _ => None,
        })
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

/// Which item each name was taken to mean.
type Means<O> = Vec<(Name, <O as Ontology>::Item)>;

/// A reading after resolution and answering.
struct Candidate<O: Ontology> {
    reading: usize,
    /// Tokens the reading takes to mean nothing.
    idle: usize,
    meaning: Utterance<O>,
    means: Means<O>,
    result: WorldResult<O>,
}

impl<O: Ontology> Mind<'_, O> {
    /// Responds to one user turn and records it in the session.
    pub fn respond(&self, session: &mut Session<O>, text: &str) -> Turn<O> {
        let tokens = self.language.tokens(text);
        let mut trace = Trace::new(text, &tokens);
        let (reply, meaning) = self.think(session, text, &mut trace);
        let said = self.language.say(&reply).unwrap_or_else(|| Spoken {
            text: format!("(I have no words for this reply: {reply:?})"),
            rules: Vec::new(),
        });
        trace.reply = Some(reply.clone());
        trace.reply_rules = said.rules;
        let spoken = match stuck_at(&reply) {
            Some(at) => format!("{}\n  {}", said.text, mark(text, &tokens, at)),
            None => said.text,
        };
        session.turns.push(TurnRecord {
            text: text.to_owned(),
            meaning,
            reply: reply.clone(),
            spoken: spoken.clone(),
        });
        Turn {
            progress: progress_of(&reply),
            understood: stuck_at(&reply).unwrap_or(tokens.len()),
            reply,
            spoken,
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
            // A correction asks the corrected request, if that rule is on.
            let meaning = match &reading.meaning {
                Utterance::Correct(thing)
                    if self.using(CORRECTION_REPLACES, &mut rt.world_rules).is_ok() =>
                {
                    session.corrected(thing)
                }
                _ => None,
            }
            .unwrap_or_else(|| reading.meaning.clone());
            for (meaning, means) in self.resolve(session, &meaning, &mut rt) {
                let fired = &mut rt.world_rules;
                let result = match &meaning {
                    Utterance::Ask(q) => self.world.answer(q, fired),
                    Utterance::Command(a) => self.world.propose(a, fired),
                    Utterance::AskReason => self
                        .using(REASON_IS_REQUEST, fired)
                        .and_then(|()| reason(session)),
                    Utterance::Forget => Ok(Reply::Forgotten),
                    Utterance::AskRecall => self
                        .using(RECALL_IS_MENTIONS, fired)
                        .map(|()| Reply::Recall(session.mentions())),
                    Utterance::Tell(f) => self.world.note(f, fired),
                    Utterance::Correct(_) => Err(Trouble::Unsupported),
                };
                rt.outcomes.push((meaning.clone(), result.clone()));
                candidates.push(Candidate {
                    reading: i,
                    idle: reading.idle,
                    meaning,
                    means,
                    result,
                });
            }
            trace.readings.push(rt);
        }
        let (reply, chosen) = self.choose(session, &candidates, trace);
        trace.chosen = chosen.map(|c| c.reading);
        (reply, chosen.map(|c| c.meaning.clone()))
    }

    /// Pushes `rule` onto `fired` if it is switched on; a meaning that needs
    /// a rule that is switched off is unsupported.
    fn using(&self, rule: RuleInfo, fired: &mut Vec<RuleInfo>) -> Result<(), Trouble<O>> {
        if !self.switches.on(rule.id) {
            return Err(Trouble::Unsupported);
        }
        fired.push(rule);
        Ok(())
    }

    /// Every way of resolving the reading's names to items, each with the
    /// item each name was taken to mean. A name with no item yields one
    /// unresolved candidate, which the world rejects. The world rules used
    /// to look the names up go into `rt`.
    fn resolve(
        &self,
        session: &Session<O>,
        meaning: &Utterance<O>,
        rt: &mut ReadingTrace<O>,
    ) -> Vec<(Utterance<O>, Means<O>)> {
        let things = meaning.things();
        let mut options = Vec::new();
        for &thing in &things {
            let mut choices = vec![thing.clone()];
            if let Thing::Named(name) = thing {
                // A name the user gave an item means it, if that rule is on.
                let items = match session.aliased(name) {
                    Some(item) if self.using(ALIAS_NOTED, &mut rt.world_rules).is_ok() => {
                        vec![item]
                    }
                    _ => self.world.lookup(name, &mut rt.world_rules),
                };
                rt.resolutions.push((name.clone(), items.clone()));
                if !items.is_empty() {
                    choices = items.into_iter().map(Thing::Item).collect();
                }
            } else if *thing == Thing::Previous {
                // The one item the latest turn asked about, if that rule is on.
                if let Some(item) = session.topic()
                    && self.using(PREVIOUS_IS_TOPIC, &mut rt.world_rules).is_ok()
                {
                    choices = vec![Thing::Item(item)];
                }
            }
            options.push(choices);
        }
        cartesian(&options)
            .into_iter()
            .map(|combo| {
                let mut chosen = combo.iter();
                let resolved = meaning.map_things(&mut |t| chosen.next().unwrap_or(t).clone());
                let means = things
                    .iter()
                    .zip(&combo)
                    .filter_map(|pair| match pair {
                        (Thing::Named(n), Thing::Item(i)) => Some((n.clone(), i.clone())),
                        _ => None,
                    })
                    .collect();
                (resolved, means)
            })
            .collect()
    }

    /// Picks the reply from the candidates, by the preference rules, and
    /// the candidate it came from.
    fn choose<'c>(
        &self,
        session: &Session<O>,
        candidates: &'c [Candidate<O>],
        trace: &mut Trace<O>,
    ) -> (Reply<O>, Option<&'c Candidate<O>>) {
        let mut good: Vec<(&Candidate<O>, &Reply<O>)> = candidates
            .iter()
            .filter_map(|c| Some((c, c.result.as_ref().ok()?)))
            .collect();
        let replies: BTreeSet<&Reply<O>> = good.iter().map(|&(_, r)| r).collect();
        let least = good.iter().map(|(c, _)| c.idle).min();
        if replies.len() > 1 && self.switches.on(PREFER_FEWEST_IDLE.id) {
            good.retain(|(c, _)| Some(c.idle) == least);
            trace.preferences.push(PREFER_FEWEST_IDLE);
        }
        let Some(&(first, first_reply)) = good.first() else {
            // Nothing survived: report the trouble of the reading that got
            // furthest (the earliest of those, which understood the most
            // of what was said), or ask back.
            let asking = self.switches.on(PREFER_ASKING.id);
            let asks = |t: &Trouble<O>| asking && matches!(t, Trouble::Ambiguous { .. });
            let referring = self.switches.on(PREFER_REFERRING.id);
            let refers =
                |c: &Candidate<O>| !referring || !c.meaning.things().contains(&&Thing::Previous);
            let worst = candidates
                .iter()
                .rev()
                .filter_map(|c| Some((c, c.result.as_ref().err()?)))
                .max_by_key(|&(c, t)| (asks(t), refers(c), trouble_progress(t)));
            let Some((c, trouble)) = worst else {
                return (Reply::Trouble(Trouble::Unsupported), None);
            };
            if asks(trouble) {
                trace.preferences.push(PREFER_ASKING);
            }
            if referring && candidates.iter().any(|c| !refers(c)) {
                trace.preferences.push(PREFER_REFERRING);
            }
            return (Reply::Trouble(trouble.clone()), Some(c));
        };
        let replies: BTreeSet<&Reply<O>> = good.iter().map(|&(_, r)| r).collect();
        if replies.len() == 1 {
            trace.choice = format!(
                "{} surviving reading(s), all with the same answer",
                good.len()
            );
            return (first_reply.clone(), Some(first));
        }
        // The readings disagree: prefer recent mentions, then the first reading.
        let mentions = session.mentions();
        let recency = |c: &Candidate<O>| {
            c.means
                .iter()
                .filter_map(|(_, item)| mentions.iter().position(|m| m == item))
                .min()
        };
        let recent = good
            .iter()
            .filter_map(|&(c, r)| Some((recency(c)?, c, r)))
            .min_by_key(|&(recency, c, _)| (recency, c.reading));
        let (pick, reply, rule) = match recent {
            Some((_, c, r)) if self.switches.on(PREFER_RECENT_MENTION.id) => {
                (c, r, PREFER_RECENT_MENTION)
            }
            _ if self.switches.on(PREFER_FIRST_READING.id) => {
                (first, first_reply, PREFER_FIRST_READING)
            }
            _ => {
                let candidates = good
                    .iter()
                    .flat_map(|(c, _)| c.means.iter().map(|(_, i)| i.clone()))
                    .collect();
                return (Reply::Trouble(Trouble::Ambiguous { candidates }), None);
            }
        };
        trace.preferences.push(rule);
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
                    .any(|(c, _)| c.means.iter().any(|(n, i)| n == name && i != item))
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
        (
            Reply::Assuming {
                assumptions,
                reply: Box::new(reply.clone()),
            },
            Some(pick),
        )
    }
}

/// The items the user spoke of in a turn, as it was understood.
fn spoken_of<O: Ontology>(turn: &TurnRecord<O>) -> impl Iterator<Item = &O::Item> {
    turn.meaning
        .iter()
        .flat_map(Utterance::things)
        .filter_map(Thing::item)
}

/// Why the most recent action asked for was taken up: the turn that asked.
fn reason<O: Ontology>(session: &Session<O>) -> Result<Reply<O>, Trouble<O>> {
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

/// Every way of taking one choice from each of `options`, in order; at most
/// 256, a guard against inputs with many ambiguous names.
fn cartesian<T: Clone>(options: &[Vec<T>]) -> Vec<Vec<T>> {
    options.iter().fold(vec![Vec::new()], |out, choices| {
        let mut next: Vec<Vec<T>> = out
            .iter()
            .flat_map(|prefix| {
                choices
                    .iter()
                    .map(|c| prefix.iter().chain([c]).cloned().collect())
            })
            .collect();
        next.truncate(256);
        next
    })
}

/// The token where reading stopped, if the reply is that the input was not
/// understood.
fn stuck_at<O: Ontology>(reply: &Reply<O>) -> Option<usize> {
    match reply {
        Reply::Trouble(Trouble::UnknownWord { at, .. } | Trouble::UnknownShape { at }) => Some(*at),
        _ => None,
    }
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
        _ => Progress::Done,
    }
}
