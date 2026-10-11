//! A deterministic event-sourced story world. No benchmark task IDs or answer keys
//! enter the reasoner. Unknown assertions taint a story until reset.
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;
mod policy;
pub use policy::{ActionKind, ActionModel};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Entity(pub String);
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Location {
    pub possible: BTreeSet<Entity>,
    pub excluded: BTreeSet<Entity>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Locate {
        actors: Vec<Entity>,
        places: Vec<Entity>,
        negative: bool,
    },
    Take {
        actor: Entity,
        object: Entity,
    },
    Drop {
        actor: Entity,
        object: Entity,
    },
    Give {
        actor: Entity,
        object: Entity,
        recipient: Entity,
    },
    Relation {
        subject: Entity,
        relation: String,
        object: Entity,
    },
    Kind {
        entity: Entity,
        kind: Entity,
    },
    Rule {
        kind: Entity,
        relation: String,
        value: Entity,
    },
    TimedLocate {
        actor: Entity,
        place: Entity,
        time: TimePoint,
    },
    Attribute {
        entity: Entity,
        attribute: String,
        value: Entity,
    },
    State {
        actor: Entity,
        state: Entity,
    },
}
/// Explicit discourse times sort independently of sentence order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TimePoint {
    Yesterday,
    Morning,
    Afternoon,
    Evening,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Reply {
    pub answer: Option<String>,
    pub reason: String,
}
impl Reply {
    fn known(s: impl Into<String>) -> Self {
        Self {
            answer: Some(s.into()),
            reason: "derived from story state".into(),
        }
    }
    fn refuse(s: impl Into<String>) -> Self {
        Self {
            answer: None,
            reason: s.into(),
        }
    }
}
#[derive(Default, Debug)]
pub struct World {
    pub events: Vec<Event>,
    locations: BTreeMap<Entity, Location>,
    holders: BTreeMap<Entity, Entity>,
    history: BTreeMap<Entity, Vec<Entity>>,
    relations: BTreeMap<(Entity, String), BTreeSet<Entity>>,
    kinds: BTreeMap<Entity, Entity>,
    rules: BTreeMap<(Entity, String), Entity>,
    timeline: BTreeMap<Entity, BTreeMap<TimePoint, Entity>>,
    attributes: BTreeMap<(Entity, String), Entity>,
    states: BTreeMap<Entity, Entity>,
    model: Arc<ActionModel>,
    action_evidence: Vec<(Entity, ActionKind, Entity, Entity)>,
    focus: Vec<Entity>,
    object_focus: Option<Entity>,
    tainted: bool,
}
fn e(s: &str) -> Entity {
    Entity(s.trim().trim_start_matches("the ").to_string())
}
fn captures(pattern: &'static str, s: &str) -> Option<Vec<String>> {
    thread_local! {static CACHE: RefCell<BTreeMap<&'static str, Regex>> = const { RefCell::new(BTreeMap::new()) };}
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if !cache.contains_key(pattern) {
            cache.insert(pattern, Regex::new(pattern).ok()?);
        }
        let c = cache.get(pattern)?.captures(s)?;
        Some(
            c.iter()
                .skip(1)
                .map(|v| v.map_or("", |m| m.as_str()).to_string())
                .collect(),
        )
    })
}
fn singular(s: &str) -> &str {
    match s {
        "mice" => "mouse",
        "wolves" => "wolf",
        "sheep" => "sheep",
        _ => s.strip_suffix('s').unwrap_or(s),
    }
}
fn inverse(r: &str) -> Option<&str> {
    Some(match r {
        "north" => "south",
        "south" => "north",
        "east" => "west",
        "west" => "east",
        "left" => "right",
        "right" => "left",
        "above" => "below",
        "below" => "above",
        "bigger" => "smaller",
        "smaller" => "bigger",
        _ => return None,
    })
}
impl World {
    pub fn with_model(model: Arc<ActionModel>) -> Self {
        Self {
            model,
            ..Self::default()
        }
    }
    fn actors(&self, s: &str) -> Option<Vec<Entity>> {
        match s {
            "he" | "she" => (self.focus.len() == 1).then(|| self.focus.clone()),
            "they" => (!self.focus.is_empty()).then(|| self.focus.clone()),
            "it" => Some(vec![self.object_focus.clone()?]),
            _ => Some(s.split(" and ").map(e).collect()),
        }
    }
    fn object(&self, text: &str) -> Option<Entity> {
        if text == "it" {
            self.object_focus.clone()
        } else {
            Some(e(text))
        }
    }
    pub fn parse(&self, text: &str) -> Option<Event> {
        let s = text.trim().trim_end_matches('.').to_lowercase();
        let s = s
            .strip_prefix("after that ")
            .or_else(|| s.strip_prefix("following that "))
            .or_else(|| s.strip_prefix("afterwards "))
            .or_else(|| s.strip_prefix("then "))
            .unwrap_or(&s);
        for (phrase, time) in [
            ("yesterday", TimePoint::Yesterday),
            ("this morning", TimePoint::Morning),
            ("this afternoon", TimePoint::Afternoon),
            ("this evening", TimePoint::Evening),
        ] {
            let bare = s
                .strip_prefix(&format!("{phrase} "))
                .or_else(|| s.strip_suffix(&format!(" {phrase}")));
            if let Some(bare) = bare {
                if let Some(Event::Locate {
                    actors,
                    places,
                    negative: false,
                }) = self.parse(bare)
                    && actors.len() == 1
                    && places.len() == 1
                {
                    return Some(Event::TimedLocate {
                        actor: actors.first()?.clone(),
                        place: places.first()?.clone(),
                        time,
                    });
                }
                return None;
            }
        }
        if let Some(c) = captures(
            r"^(.+?) (?:went|moved|travelled|journeyed|traveled) (?:back )?to (?:the )?(.+)$",
            s,
        ) && let [a, p] = c.as_slice()
        {
            return Some(Event::Locate {
                actors: self.actors(a)?,
                places: vec![e(p)],
                negative: false,
            });
        }
        if let Some(c) = captures(
            r"^(.+?) (?:is|was) (not |no longer )?(?:in|at) (?:the )?(.+)$",
            s,
        ) && let [a, n, p] = c.as_slice()
        {
            return Some(Event::Locate {
                actors: self.actors(a)?,
                places: p.replace("either ", "").split(" or ").map(e).collect(),
                negative: !n.is_empty(),
            });
        }
        if let Some(c) = captures(
            r"^(.+?) (?:is|was) (?:either in|either at) (?:the )?(.+)$",
            s,
        ) && let [a, p] = c.as_slice()
        {
            return Some(Event::Locate {
                actors: self.actors(a)?,
                places: p.split(" or ").map(e).collect(),
                negative: false,
            });
        }
        // Copular relations precede action verbs: "to the left of" is not dropping an object.
        if let Some(c) = captures(
            r"^(?:the )?(.+?) is (?:to the )?(north|south|east|west|left|right|above|below|bigger|smaller)(?: of| than)? (?:the )?(.+)$",
            s,
        ) && let [a, r, b] = c.as_slice()
        {
            return Some(Event::Relation {
                subject: e(a),
                relation: r.clone(),
                object: e(b),
            });
        }
        if let Some(c) = captures(
            r"^(.+?) (picked up|took|grabbed|got|discarded|dropped|put down|left) (?:the )?(.+?)(?: there)?$",
            s,
        ) && let [a, v, o] = c.as_slice()
        {
            let actors = self.actors(a)?;
            let actor = actors.first()?.clone();
            if actors.len() != 1 {
                return None;
            }
            return Some(
                if matches!(v.as_str(), "discarded" | "dropped" | "put down" | "left") {
                    Event::Drop {
                        actor,
                        object: self.object(o)?,
                    }
                } else {
                    Event::Take {
                        actor,
                        object: self.object(o)?,
                    }
                },
            );
        }
        if let Some(c) = captures(r"^(.+?) (?:gave|handed|passed) (?:the )?(.+?) to (.+)$", s)
            && let [a, o, b] = c.as_slice()
        {
            return Some(Event::Give {
                actor: self.actors(a)?.first()?.clone(),
                object: self.object(o)?,
                recipient: self.actors(b)?.first()?.clone(),
            });
        }
        if let Some(c) = captures(r"^(?:the )?(.+?) fits (?:inside|in) (?:the )?(.+)$", s)
            && let [a, b] = c.as_slice()
        {
            return Some(Event::Relation {
                subject: e(a),
                relation: "smaller".into(),
                object: e(b),
            });
        }
        if let Some(c) = captures(r"^(?:the )?(.+?) (?:is a|is an) (.+)$", s)
            && let [a, k] = c.as_slice()
        {
            return Some(Event::Kind {
                entity: e(a),
                kind: e(k),
            });
        }
        if let Some(c) = captures(r"^(.+?) (?:are afraid of|fear) (.+)$", s)
            && let [k, v] = c.as_slice()
        {
            return Some(Event::Rule {
                kind: e(singular(k)),
                relation: "fear".into(),
                value: e(singular(v)),
            });
        }
        if let Some(c) = captures(r"^(.+?) (?:is afraid of|fears) (.+)$", s)
            && let [a, b] = c.as_slice()
        {
            return Some(Event::Relation {
                subject: e(a),
                relation: "fear".into(),
                object: e(b),
            });
        }
        if let Some(c) = captures(
            r"^(.+?) is (red|green|blue|yellow|white|black|gray|grey|pink|purple|orange|brown)$",
            s,
        ) && let [a, v] = c.as_slice()
        {
            return Some(Event::Attribute {
                entity: e(a),
                attribute: "color".into(),
                value: e(v),
            });
        }
        if let Some(c) = captures(r"^(.+?) is ([a-z]+)$", s)
            && let [a, state] = c.as_slice()
        {
            return Some(Event::State {
                actor: e(a),
                state: e(state),
            });
        }
        None
    }
    fn locate(&mut self, entity: &Entity, loc: Location) {
        if loc.possible.len() == 1
            && let Some(p) = loc.possible.first()
        {
            let h = self.history.entry(entity.clone()).or_default();
            if h.last() != Some(p) {
                h.push(p.clone());
            }
        }
        self.locations.insert(entity.clone(), loc);
    }
    /// Moving a holder changes the location of every object still in their custody.
    fn locate_with_inventory(&mut self, actor: &Entity, loc: Location) {
        self.locate(actor, loc.clone());
        let held: Vec<_> = self
            .holders
            .iter()
            .filter(|(_, h)| *h == actor)
            .map(|(o, _)| o.clone())
            .collect();
        for object in held {
            self.locate(&object, loc.clone());
        }
    }
    pub fn apply(&mut self, event: Event) {
        for (actor, kind, target) in policy::actions(&event) {
            if let Some(state) = self.states.get(&actor) {
                self.action_evidence
                    .push((actor, kind, target, state.clone()));
            }
        }
        match &event {
            Event::Locate {
                actors,
                places,
                negative,
            } => {
                for a in actors {
                    let mut loc = if *negative {
                        self.locations.get(a).cloned().unwrap_or_default()
                    } else {
                        Location::default()
                    };
                    for p in places {
                        if *negative {
                            loc.possible.remove(p);
                            loc.excluded.insert(p.clone());
                        } else {
                            loc.possible.insert(p.clone());
                        }
                    }
                    self.locate_with_inventory(a, loc);
                }
                self.focus = actors.clone();
            }
            Event::Take { actor, object } => {
                self.object_focus = Some(object.clone());
                self.holders.insert(object.clone(), actor.clone());
                self.locate(
                    object,
                    self.locations.get(actor).cloned().unwrap_or_default(),
                );
                self.focus = vec![actor.clone()];
            }
            Event::Drop { actor, object } => {
                self.object_focus = Some(object.clone());
                self.holders.remove(object);
                self.locate(
                    object,
                    self.locations.get(actor).cloned().unwrap_or_default(),
                );
                self.focus = vec![actor.clone()];
            }
            Event::Give {
                actor,
                object,
                recipient,
            } => {
                self.object_focus = Some(object.clone());
                self.holders.insert(object.clone(), recipient.clone());
                // A transfer establishes co-location even when the recipient had no location.
                let loc = self
                    .locations
                    .get(actor)
                    .filter(|loc| !loc.possible.is_empty())
                    .or_else(|| self.locations.get(recipient))
                    .cloned()
                    .unwrap_or_default();
                self.locate_with_inventory(actor, loc.clone());
                self.locate_with_inventory(recipient, loc);
                self.focus = vec![actor.clone()];
            }
            Event::Relation {
                subject,
                relation,
                object,
            } => {
                self.relations
                    .entry((subject.clone(), relation.clone()))
                    .or_default()
                    .insert(object.clone());
                if let Some(r) = inverse(relation) {
                    self.relations
                        .entry((object.clone(), r.into()))
                        .or_default()
                        .insert(subject.clone());
                }
            }
            Event::Kind { entity, kind } => {
                self.kinds.insert(entity.clone(), kind.clone());
            }
            Event::Rule {
                kind,
                relation,
                value,
            } => {
                self.rules
                    .insert((kind.clone(), relation.clone()), value.clone());
            }
            Event::TimedLocate { actor, place, time } => {
                let timeline = self.timeline.entry(actor.clone()).or_default();
                timeline.insert(*time, place.clone());
                let history: Vec<_> = timeline.values().cloned().collect();
                let latest = history.last().cloned();
                self.history.insert(actor.clone(), history);
                if let Some(latest) = latest {
                    self.locations.insert(
                        actor.clone(),
                        Location {
                            possible: BTreeSet::from([latest]),
                            excluded: BTreeSet::new(),
                        },
                    );
                }
                self.focus = vec![actor.clone()];
            }
            Event::Attribute {
                entity,
                attribute,
                value,
            } => {
                self.attributes
                    .insert((entity.clone(), attribute.clone()), value.clone());
            }
            Event::State { actor, state } => {
                self.states.insert(actor.clone(), state.clone());
                self.focus = vec![actor.clone()];
            }
        }
        self.events.push(event);
    }
    pub fn observe(&mut self, text: &str) -> Reply {
        if let Some(event) = self.parse(text) {
            self.apply(event);
            Reply {
                answer: None,
                reason: "event accepted".into(),
            }
        } else {
            self.tainted = true;
            Reply::refuse("unsupported assertion; reset required before answering")
        }
    }
    fn reachable(&self, from: &Entity, relation: &str) -> BTreeSet<Entity> {
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::from([from.clone()]);
        while let Some(x) = queue.pop_front() {
            if let Some(next) = self.relations.get(&(x, relation.into())) {
                for y in next {
                    if seen.insert(y.clone()) {
                        queue.push_back(y.clone());
                    }
                }
            }
        }
        seen
    }
    fn one(values: BTreeSet<Entity>) -> Reply {
        if values.len() == 1 {
            values
                .first()
                .map_or_else(|| Reply::refuse("no evidence"), |v| Reply::known(&v.0))
        } else {
            Reply::refuse("missing or ambiguous evidence")
        }
    }
    /// Compose axis-aligned spatial constraints, refusing inconsistent graphs.
    fn displacement(&self, from: &Entity, to: &Entity) -> Option<(i32, i32)> {
        let mut positions = BTreeMap::from([(from.clone(), (0i32, 0i32))]);
        let mut queue = VecDeque::from([from.clone()]);
        while let Some(at) = queue.pop_front() {
            let (x, y) = *positions.get(&at)?;
            for (relation, dx, dy) in [
                ("north", 0, -1),
                ("south", 0, 1),
                ("east", -1, 0),
                ("west", 1, 0),
                ("above", 0, -1),
                ("below", 0, 1),
                ("right", -1, 0),
                ("left", 1, 0),
            ] {
                if let Some(next) = self.relations.get(&(at.clone(), relation.into())) {
                    for n in next {
                        let position = (x.checked_add(dx)?, y.checked_add(dy)?);
                        if let Some(old) = positions.get(n) {
                            if *old != position {
                                return None;
                            }
                        } else {
                            positions.insert(n.clone(), position);
                            queue.push_back(n.clone());
                        }
                    }
                }
            }
        }
        positions.get(to).copied()
    }
    pub fn ask(&self, text: &str) -> Reply {
        if self.tainted {
            return Reply::refuse("story contains unsupported assertions");
        }
        let s = text.trim().trim_end_matches('?').to_lowercase();
        let mut words = Vec::new();
        for word in s.split_whitespace() {
            if word == "it" {
                let Some(object) = &self.object_focus else {
                    return Reply::refuse("unresolved object pronoun");
                };
                words.push(object.0.as_str());
            } else {
                words.push(word);
            }
        }
        let s = words.join(" ");
        let s = s.trim_end_matches(" now");
        if let Some(c) = captures(r"^where will (.+?) go$", s)
            && let [a] = c.as_slice()
        {
            if let Some(state) = self.states.get(&e(a))
                && let Some(target) = self.model.predict(state, ActionKind::Destination)
            {
                return Reply {
                    answer: Some(target.0.clone()),
                    reason: "predicted from unanimous training state/action observations".into(),
                };
            }
            return Reply::refuse("no reliable learned action for this state");
        }
        if let Some(c) = captures(r"^why did (.+?) (go to|get) (?:the )?(.+)$", s)
            && let [a, v, target] = c.as_slice()
        {
            let kind = if v == "go to" {
                ActionKind::Destination
            } else {
                ActionKind::Acquisition
            };
            for (actor, action, object, state) in self.action_evidence.iter().rev() {
                if actor == &e(a) && *action == kind && object == &e(target) {
                    if self.model.predict(state, kind) == Some(object) {
                        return Reply {
                            answer: Some(state.0.clone()),
                            reason:
                                "inferred motive: prior state and learned state/action association"
                                    .into(),
                        };
                    }
                    break;
                }
            }
            return Reply::refuse("motive not supported by prior state and learned action");
        }
        if let Some(c) = captures(r"^what colou?r is (.+)$", s)
            && let [a] = c.as_slice()
        {
            if let Some(v) = self.attributes.get(&(e(a), "color".into())) {
                return Reply::known(&v.0);
            }
            if let Some(kind) = self.kinds.get(&e(a)) {
                let values: BTreeSet<_> = self
                    .kinds
                    .iter()
                    .filter(|(_, k)| *k == kind)
                    .filter_map(|(entity, _)| {
                        self.attributes
                            .get(&(entity.clone(), "color".into()))
                            .cloned()
                    })
                    .collect();
                let mut reply = Self::one(values);
                if reply.answer.is_some() {
                    reply.reason =
                        "induced from unanimous observed members of the same kind; defeasible"
                            .into();
                }
                return reply;
            }
            return Reply::refuse("no color evidence");
        }
        if let Some(c) = captures(r"^where (?:is|was) (?:the )?(.+?) before (?:the )?(.+)$", s)
            && let [a, p] = c.as_slice()
        {
            if let Some(h) = self.history.get(&e(a)) {
                for pair in h.windows(2).rev() {
                    if let [before, after] = pair
                        && after == &e(p)
                    {
                        return Reply::known(&before.0);
                    }
                }
            }
            return Reply::refuse("no earlier location");
        }
        if let Some(c) = captures(r"^where (?:is|was) (?:the )?(.+)$", s)
            && let [a] = c.as_slice()
        {
            return self.locations.get(&e(a)).map_or_else(
                || Reply::refuse("unknown location"),
                |l| Self::one(l.possible.clone()),
            );
        }
        if let Some(c) = captures(r"^is (.+?) (?:in|at) (?:the )?(.+)$", s)
            && let [a, p] = c.as_slice()
        {
            return match self.locations.get(&e(a)) {
                Some(l) if l.possible.len() == 1 && l.possible.contains(&e(p)) => {
                    Reply::known("yes")
                }
                Some(l)
                    if l.excluded.contains(&e(p))
                        || (!l.possible.is_empty() && !l.possible.contains(&e(p))) =>
                {
                    Reply::known("no")
                }
                _ => Reply::known("maybe"),
            };
        }
        if let Some(c) = captures(
            r"^(how many objects is|what is) (.+?) (?:carrying|holding)$",
            s,
        ) && let [q, a] = c.as_slice()
        {
            if !self.locations.contains_key(&e(a))
                && !self.events.iter().any(
                    |ev| matches!(ev,Event::Take{actor,..}|Event::Drop{actor,..} if actor==&e(a)),
                )
            {
                return Reply::refuse("unknown person");
            }
            let held: Vec<_> = self
                .holders
                .iter()
                .filter(|(_, h)| *h == &e(a))
                .map(|(o, _)| o.0.clone())
                .collect();
            if q.starts_with("how") {
                let n = match held.len() {
                    0 => "none".into(),
                    1 => "one".into(),
                    2 => "two".into(),
                    3 => "three".into(),
                    4 => "four".into(),
                    5 => "five".into(),
                    n => n.to_string(),
                };
                return Reply::known(n);
            }
            return Reply::known(if held.is_empty() {
                "nothing".into()
            } else {
                held.join(",")
            });
        }
        if let Some(c) = captures(r"^who (?:has|is carrying|is holding) (?:the )?(.+)$", s)
            && let [o] = c.as_slice()
        {
            return self
                .holders
                .get(&e(o))
                .map_or_else(|| Reply::refuse("unknown holder"), |a| Reply::known(&a.0));
        }
        if let Some(c) = captures(r"^who (received|gave) (?:the )?(.+)$", s)
            && let [v, o] = c.as_slice()
        {
            for ev in self.events.iter().rev() {
                if let Event::Give {
                    actor,
                    object,
                    recipient,
                } = ev
                    && object == &e(o)
                {
                    return Reply::known(if v == "received" {
                        &recipient.0
                    } else {
                        &actor.0
                    });
                }
            }
        }
        if let Some(c) = captures(r"^who did (.+?) (?:get|receive) (?:the )?(.+?) from$", s)
            && let [a, o] = c.as_slice()
        {
            for ev in self.events.iter().rev() {
                if let Event::Give {
                    actor,
                    object,
                    recipient,
                } = ev
                    && recipient == &e(a)
                    && object == &e(o)
                {
                    return Reply::known(&actor.0);
                }
            }
        }
        if let Some(c) = captures(r"^who did (.+?) (?:give|hand|pass) (?:the )?(.+?) to$", s)
            && let [a, o] = c.as_slice()
        {
            for ev in self.events.iter().rev() {
                if let Event::Give {
                    actor,
                    object,
                    recipient,
                } = ev
                    && actor == &e(a)
                    && object == &e(o)
                {
                    return Reply::known(&recipient.0);
                }
            }
        }
        if let Some(c) = captures(r"^who (?:gave|handed|passed) (?:the )?(.+?) to (.+)$", s)
            && let [o, b] = c.as_slice()
        {
            for ev in self.events.iter().rev() {
                if let Event::Give {
                    actor,
                    object,
                    recipient,
                } = ev
                    && object == &e(o)
                    && recipient == &e(b)
                {
                    return Reply::known(&actor.0);
                }
            }
        }
        if let Some(c) = captures(r"^what did (.+?) (?:give|hand|pass) to (.+)$", s)
            && let [a, b] = c.as_slice()
        {
            for ev in self.events.iter().rev() {
                if let Event::Give {
                    actor,
                    object,
                    recipient,
                } = ev
                    && actor == &e(a)
                    && recipient == &e(b)
                {
                    return Reply::known(&object.0);
                }
            }
        }
        if let Some(c) = captures(r"^what is (.+?) afraid of$", s)
            && let [a] = c.as_slice()
        {
            if let Some(k) = self.kinds.get(&e(a))
                && let Some(v) = self.rules.get(&(k.clone(), "fear".into()))
            {
                return Reply::known(&v.0);
            }
            return Self::one(
                self.relations
                    .get(&(e(a), "fear".into()))
                    .cloned()
                    .unwrap_or_default(),
            );
        }
        if let Some(c) = captures(
            r"^what is (?:the )?(.+?) (north|south|east|west|left|right|above|below) of$",
            s,
        ) && let [a, r] = c.as_slice()
        {
            return Self::one(
                self.relations
                    .get(&(e(a), r.clone()))
                    .cloned()
                    .unwrap_or_default(),
            );
        }
        if let Some(c) = captures(
            r"^what is (?:to the )?(north|south|east|west|left|right|above|below)(?: of)? (?:the )?(.+)$",
            s,
        ) && let [r, b] = c.as_slice()
            && let Some(inv) = inverse(r)
        {
            return Self::one(
                self.relations
                    .get(&(e(b), inv.into()))
                    .cloned()
                    .unwrap_or_default(),
            );
        }
        if let Some(c) = captures(r"^does (?:the )?(.+?) fit (?:inside|in) (?:the )?(.+)$", s)
            && let [a, b] = c.as_slice()
        {
            if self.reachable(&e(a), "smaller").contains(&e(b)) {
                return Reply::known("yes");
            }
            if self.reachable(&e(a), "bigger").contains(&e(b)) {
                return Reply::known("no");
            }
            return Reply::refuse("relative size unproved");
        }
        if let Some(c) = captures(
            r"^is (?:the )?(.+?) (?:to the )?(north|south|east|west|left|right|above|below|bigger|smaller)(?: of| than)? (?:the )?(.+)$",
            s,
        ) && let [a, r, b] = c.as_slice()
        {
            if !matches!(r.as_str(), "bigger" | "smaller") {
                if let Some((x, y)) = self.displacement(&e(b), &e(a)) {
                    let yes = match r.as_str() {
                        "north" | "above" => y > 0,
                        "south" | "below" => y < 0,
                        "east" | "right" => x > 0,
                        "west" | "left" => x < 0,
                        _ => false,
                    };
                    return Reply::known(if yes { "yes" } else { "no" });
                }
                return Reply::refuse("spatial relation unproved or inconsistent");
            }
            if self.reachable(&e(a), r).contains(&e(b)) {
                return Reply::known("yes");
            }
            if let Some(inv) = inverse(r)
                && self.reachable(&e(a), inv).contains(&e(b))
            {
                return Reply::known("no");
            }
            return Reply::refuse("relation unproved");
        }
        if let Some(c) = captures(r"^how do you go from (?:the )?(.+?) to (?:the )?(.+)$", s)
            && let [a, b] = c.as_slice()
        {
            let mut queue = VecDeque::from([(e(a), Vec::<String>::new())]);
            let mut seen = BTreeSet::from([e(a)]);
            while let Some((at, path)) = queue.pop_front() {
                if at == e(b) {
                    return Reply::known(path.join(","));
                }
                for (r, step) in [("north", "s"), ("south", "n"), ("east", "w"), ("west", "e")] {
                    if let Some(next) = self.relations.get(&(at.clone(), r.into())) {
                        for n in next {
                            if seen.insert(n.clone()) {
                                let mut p = path.clone();
                                p.push(step.into());
                                queue.push_back((n.clone(), p));
                            }
                        }
                    }
                }
            }
            return Reply::refuse("no path");
        }
        Reply::refuse("unsupported question")
    }
}
