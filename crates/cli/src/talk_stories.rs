//! Bounded, replayable story context for the main conversation.
use glassmind_stories::{ActionKind, ActionModel, Entity, Event, World};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

const MAX_ASSERTIONS: usize = 512;
const MAX_BYTES: usize = 262_144;

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Stories {
    #[serde(default)]
    assertions: Vec<String>,
    #[serde(default)]
    invalidated: bool,
    #[serde(skip)]
    world: World,
}

#[derive(Serialize)]
pub(crate) struct StoryReply {
    pub kind: &'static str,
    pub text: String,
    pub answer: Option<String>,
    pub reason: String,
}

/// A read-only routing decision; only `apply` changes persisted story state.
pub(crate) enum StoryPlan {
    Ignore,
    Reply(StoryReply),
    Invalidate(&'static str),
    Observe(String),
}

impl StoryPlan {
    pub fn supported(&self) -> bool {
        matches!(self, Self::Reply(_) | Self::Observe(_))
    }
}

impl Stories {
    pub fn has_context(&self) -> bool {
        self.invalidated || !self.assertions.is_empty()
    }

    /// Established story actors disambiguate a salutation-shaped name in a
    /// supported coordinated event. This is state evidence, never an intent label.
    pub fn has_actor(&self, name: &str) -> bool {
        let named = |entity: &Entity| entity.0 == name;
        self.world.events.iter().any(|event| match event {
            Event::Locate { actors, .. } => actors.iter().any(named),
            Event::Take { actor, .. }
            | Event::Drop { actor, .. }
            | Event::Give { actor, .. }
            | Event::TimedLocate { actor, .. }
            | Event::State { actor, .. } => named(actor),
            Event::Kind { entity, .. } | Event::Attribute { entity, .. } => named(entity),
            Event::Relation { subject, .. } => named(subject),
            Event::Rule { .. } => false,
        })
    }

    pub fn restore(&mut self) -> Result<(), String> {
        self.world = World::with_model(action_model()?);
        if self.assertions.len() > MAX_ASSERTIONS
            || self.assertions.iter().map(String::len).sum::<usize>() > MAX_BYTES
        {
            self.assertions.clear();
            self.invalidated = true;
        }
        for line in &self.assertions {
            self.world.observe(line);
        }
        Ok(())
    }

    fn refuse(reason: &str) -> StoryReply {
        StoryReply {
            kind: "refusal",
            text: format!("I can't answer that from the facts I have. {reason}"),
            answer: None,
            reason: reason.into(),
        }
    }

    fn invalidate(&mut self, reason: &str) -> StoryReply {
        self.invalidated = true;
        // Never retain a partial history that could become confidently answerable.
        self.assertions.clear();
        Self::refuse(reason)
    }

    pub fn plan(&self, line: &str) -> StoryPlan {
        let lower = line.trim().to_lowercase();
        if lower
            .trim_end_matches(['.', '?', '!'])
            .contains(['.', '?', ';', '!'])
        {
            return StoryPlan::Invalidate(
                "Please give one statement or question at a time. Use /reset and restate the facts.",
            );
        }
        let first = lower.split_whitespace().next().unwrap_or_default();
        let question = lower.ends_with('?')
            || matches!(
                first,
                "where"
                    | "what"
                    | "who"
                    | "why"
                    | "how"
                    | "is"
                    | "does"
                    | "are"
                    | "was"
                    | "were"
                    | "do"
                    | "did"
                    | "can"
                    | "could"
                    | "would"
            );
        if question {
            // Ask even without '?' for normal conversational use. Unknown questions
            // do not alter facts; the ordinary conversation fallback handles them.
            let result = self.world.ask(line.trim().trim_end_matches(['?', '.']));
            if result.reason == "unsupported question" {
                return StoryPlan::Ignore;
            }
            if self.invalidated {
                return StoryPlan::Reply(Self::refuse(
                    "An earlier statement could change the answer, but I couldn't understand it. Use /reset and restate the facts.",
                ));
            }
            let Some(answer) = result.answer else {
                return StoryPlan::Reply(Self::refuse(&result.reason));
            };
            let derived = result.reason == "derived from story state";
            return StoryPlan::Reply(StoryReply {
                kind: if derived {
                    "story_answer"
                } else {
                    "story_prediction"
                },
                text: if derived {
                    answer.clone()
                } else {
                    format!("My best inference is {answer}. {}", result.reason)
                },
                answer: Some(answer),
                reason: result.reason,
            });
        }
        // Explicit commands are requests, not assertions about the story. Only
        // fixed syntax is trusted here, never the learned intent classifier.
        if matches!(
            first,
            "calculate"
                | "compute"
                | "convert"
                | "evaluate"
                | "book"
                | "call"
                | "remind"
                | "set"
                | "please"
                | "tell"
                | "show"
        ) {
            return StoryPlan::Ignore;
        }
        let mut statement = lower.trim();
        let mut correction = false;
        for prefix in ["actually, ", "actually ", "no, ", "correction: "] {
            if let Some(rest) = statement.strip_prefix(prefix) {
                statement = rest;
                correction = true;
                break;
            }
        }
        let event = self.world.parse(statement);
        // Give parsing in the existing reasoner chooses the first actor. Do
        // not silently discard a coordinated giver or recipient in conversation.
        let safe = event.as_ref().is_some_and(safe_event)
            && !(matches!(event, Some(Event::Give { .. })) && statement.contains(" and "));
        let replaceable = matches!(
            event,
            Some(
                Event::Locate {
                    negative: false,
                    ..
                } | Event::Attribute { .. }
                    | Event::State { .. }
                    | Event::Kind { .. }
            )
        );
        if !safe || (correction && !replaceable) {
            return StoryPlan::Invalidate(
                "I couldn't safely update the facts from that statement. Use /reset and restate the facts before asking about them.",
            );
        }
        if self.invalidated {
            return StoryPlan::Reply(Self::refuse(
                "Use /reset and restate the facts so I can answer reliably again.",
            ));
        }
        if self.assertions.len() >= MAX_ASSERTIONS
            || self.assertions.iter().map(String::len).sum::<usize>() + statement.len() > MAX_BYTES
        {
            return StoryPlan::Invalidate(
                "This conversation has reached its fact limit. Use /reset to start a new one.",
            );
        }
        StoryPlan::Observe(statement.to_owned())
    }

    pub fn apply(&mut self, plan: StoryPlan) -> Option<StoryReply> {
        match plan {
            StoryPlan::Ignore => None,
            StoryPlan::Reply(reply) => Some(reply),
            StoryPlan::Invalidate(reason) => Some(self.invalidate(reason)),
            StoryPlan::Observe(statement) => {
                self.world.observe(&statement);
                self.assertions.push(statement);
                Some(StoryReply {
                    kind: "story_fact",
                    text: "Got it. I'll use that when answering your questions.".into(),
                    answer: None,
                    reason: "event accepted".into(),
                })
            }
        }
    }

    #[cfg(test)]
    pub fn respond(&mut self, line: &str) -> Option<StoryReply> {
        let plan = self.plan(line);
        self.apply(plan)
    }
}

// The story parser accepts broad regex captures. Prevent compound statements,
// negated actions and hedges from being swallowed as an entity or destination.
fn safe_entity(entity: &Entity) -> bool {
    let words: Vec<_> = entity.0.split_whitespace().collect();
    !words.is_empty()
        && words.len() <= 3
        && words.iter().all(|w| {
            !matches!(
                *w,
                "and"
                    | "or"
                    | "but"
                    | "not"
                    | "never"
                    | "no"
                    | "then"
                    | "if"
                    | "maybe"
                    | "perhaps"
                    | "possibly"
                    | "actually"
                    | "to"
                    | "may"
                    | "might"
                    | "must"
                    | "could"
                    | "would"
                    | "should"
                    | "will"
                    | "can"
                    | "cannot"
                    | "did"
                    | "didn't"
                    | "has"
                    | "had"
                    | "said"
                    | "says"
                    | "thinks"
            ) && w
                .chars()
                .all(|c| c.is_alphabetic() || matches!(c, '-' | '\'' | '’'))
        })
}

fn safe_event(event: &Event) -> bool {
    match event {
        Event::Locate { actors, places, .. } => actors.iter().chain(places).all(safe_entity),
        Event::Take { actor, object } | Event::Drop { actor, object } => {
            safe_entity(actor) && safe_entity(object)
        }
        Event::Give {
            actor,
            object,
            recipient,
        } => [actor, object, recipient].into_iter().all(safe_entity),
        Event::Relation {
            subject, object, ..
        } => safe_entity(subject) && safe_entity(object),
        Event::Kind { entity, kind } => safe_entity(entity) && safe_entity(kind),
        Event::Rule { kind, value, .. } => safe_entity(kind) && safe_entity(value),
        Event::TimedLocate { actor, place, .. } => safe_entity(actor) && safe_entity(place),
        Event::Attribute { entity, value, .. } => safe_entity(entity) && safe_entity(value),
        Event::State { actor, state } => {
            safe_entity(actor)
                && safe_entity(state)
                && action_model().is_ok_and(|model| {
                    model.predict(state, ActionKind::Destination).is_some()
                        || model.predict(state, ActionKind::Acquisition).is_some()
                })
        }
    }
}

fn action_model() -> Result<Arc<ActionModel>, String> {
    static MODEL: OnceLock<Result<Arc<ActionModel>, String>> = OnceLock::new();
    MODEL
        .get_or_init(|| {
            serde_json::from_str(include_str!("../../../tools/stories/action-model.json"))
                .map(Arc::new)
                .map_err(|e| e.to_string())
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_story() -> Result<Stories, String> {
        let mut story = Stories::default();
        story.restore()?;
        Ok(story)
    }

    #[test]
    fn repeated_plans_do_not_mutate_history_or_world() -> Result<(), String> {
        let mut story = new_story()?;
        story.respond("Elin went to the kitchen.");
        let before = serde_json::to_value(&story).unwrap();
        for _ in 0..2 {
            assert!(story.plan("Elin went to the garden.").supported());
            assert!(!story.plan("Elin teleported to the loft.").supported());
            assert_eq!(serde_json::to_value(&story).unwrap(), before);
            assert_eq!(
                story
                    .respond("Where is Elin?")
                    .and_then(|r| r.answer)
                    .as_deref(),
                Some("kitchen")
            );
        }
        let plan = story.plan("Elin went to the garden.");
        story.apply(plan);
        assert_eq!(story.assertions.len(), 2);
        assert_eq!(
            story
                .respond("Where is Elin?")
                .and_then(|r| r.answer)
                .as_deref(),
            Some("garden")
        );
        Ok(())
    }

    #[test]
    fn unsupported_updates_block_stale_answers_even_after_restart() -> Result<(), String> {
        for update in [
            "Mary teleported to the loft.",
            "Mary might have moved to the loft.",
            "Mary went to the loft and dropped the apple.",
            "Mary is gone.",
            "Mary gave the apple to John and Bill.",
            "Where is Mary? Mary teleported to the loft.",
            "Actually, the kitchen is south of the office.",
        ] {
            let mut story = new_story()?;
            story.respond("Mary went to the kitchen.");
            assert_eq!(story.respond(update).map(|r| r.kind), Some("refusal"));
            let bytes = serde_json::to_vec(&story).map_err(|e| e.to_string())?;
            let mut restored: Stories =
                serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            restored.restore()?;
            assert_eq!(
                restored.respond("Where is Mary?").and_then(|r| r.answer),
                None
            );
        }
        Ok(())
    }

    #[test]
    fn bounded_history_fails_closed() -> Result<(), String> {
        let mut story = new_story()?;
        for _ in 0..MAX_ASSERTIONS {
            assert_eq!(
                story.respond("Mary went to the kitchen.").map(|r| r.kind),
                Some("story_fact")
            );
        }
        assert_eq!(
            story.respond("Mary went to the garden.").map(|r| r.kind),
            Some("refusal")
        );
        assert!(story.assertions.is_empty());
        let bytes = serde_json::to_vec(&story).map_err(|e| e.to_string())?;
        let mut restored: Stories = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        restored.restore()?;
        assert_eq!(
            restored.respond("Where is Mary?").and_then(|r| r.answer),
            None
        );
        Ok(())
    }

    #[test]
    fn replay_preserves_pronouns_and_has_deterministic_bytes() -> Result<(), String> {
        let mut story = new_story()?;
        story.respond("Mary went to the kitchen.");
        story.respond("Mary picked up the apple.");
        let bytes = serde_json::to_vec(&story).map_err(|e| e.to_string())?;
        let mut restored: Stories = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        restored.restore()?;
        assert_eq!(
            bytes,
            serde_json::to_vec(&restored).map_err(|e| e.to_string())?
        );
        restored.respond("She went to the garden.");
        assert_eq!(
            restored
                .respond("Where is it?")
                .and_then(|r| r.answer)
                .as_deref(),
            Some("garden")
        );
        Ok(())
    }
}
