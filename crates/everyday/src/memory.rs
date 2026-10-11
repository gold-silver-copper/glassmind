//! Small explicit conversation state. Only direct first-person statements
//! write facts; intent proposals never fabricate personal information.
use crate::model::{IntentModel, Prediction};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RememberedValue {
    pub value: String,
    pub stated_on_turn: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Slot {
    Name,
    Preference(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub slot: Slot,
    pub previous: RememberedValue,
    pub replacement: RememberedValue,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Conversation {
    pub turn: u64,
    pub name: Option<RememberedValue>,
    pub preferences: BTreeMap<String, RememberedValue>,
    pub last_slot: Option<Slot>,
    pub last_user_utterance: Option<String>,
    /// Most recent correction, with provenance, rather than unlimited history.
    pub last_correction: Option<Correction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Reply {
    Remembered {
        text: String,
        slot: Slot,
    },
    Recalled {
        text: String,
        source_turn: u64,
    },
    Missing {
        text: String,
    },
    Social {
        text: String,
    },
    Intent {
        text: String,
        proposal: Prediction,
    },
    Refusal {
        text: String,
        proposal: Option<Prediction>,
    },
}

impl Reply {
    pub fn text(&self) -> &str {
        match self {
            Self::Remembered { text, .. }
            | Self::Recalled { text, .. }
            | Self::Missing { text }
            | Self::Social { text }
            | Self::Intent { text, .. }
            | Self::Refusal { text, .. } => text,
        }
    }
}

fn tidy(text: &str) -> &str {
    text.trim().trim_end_matches(['.', '!', '?']).trim()
}

/// Match case-insensitively while preserving the user's spelling in values.
fn after<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let head = text.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix)
        .then(|| tidy(text.get(prefix.len()..).unwrap_or_default()))
        .filter(|v| !v.is_empty())
}

impl Conversation {
    pub fn respond(&mut self, utterance: &str, model: Option<&IntentModel>) -> Reply {
        self.turn = self.turn.saturating_add(1);
        let text = tidy(utterance);
        let lower = text.to_lowercase().replace('’', "'");
        let reply = self.memory_reply(text, &lower).unwrap_or_else(|| {
            if matches!(lower.as_str(), "hi" | "hello" | "hey") {
                return Reply::Social { text: self.name.as_ref().map_or_else(|| "Hello!".into(), |n| format!("Hello, {}!", n.value)) };
            }
            if matches!(lower.as_str(), "thanks" | "thank you") { return Reply::Social { text: "You're welcome.".into() }; }
            let Some(model) = model else { return Reply::Refusal { text: "I can remember your name and preferences. Load an intent model to recognize everyday requests.".into(), proposal: None }; };
            let proposal = model.predict(utterance);
            if !proposal.accepted { return Reply::Refusal { text: "I'm not confident which supported request you mean. Could you rephrase it?".into(), proposal: Some(proposal) }; }
            let response = match proposal.label.as_str() {
                "greeting" => "Hello!".into(),
                "goodbye" => "Goodbye!".into(),
                "thank_you" => "You're welcome.".into(),
                "what_is_your_name" => "I'm Glassmind.".into(),
                "are_you_a_bot" => "I'm Glassmind, a software assistant.".into(),
                "how_are_you" => "I'm ready to help. What's on your mind?".into(),
                "what_can_i_ask_you" => "I can remember your name and preferences, recall your last message, and recognize everyday requests. External services aren't connected yet.".into(),
                "user_name" => return self.recall(Slot::Name),
                "repeat" => return self.recall_utterance(),
                _ => format!("I understand this as: {}. I can identify this request, but no service is connected to carry it out.", proposal.label.replace('_', " ")),
            };
            Reply::Intent { text: response, proposal }
        });
        self.last_user_utterance = Some(utterance.to_owned());
        reply
    }

    fn memory_reply(&mut self, original: &str, lower: &str) -> Option<Reply> {
        if matches!(
            lower,
            "what did i just say"
                | "what was my last message"
                | "repeat what i said"
                | "what did i say last"
        ) {
            return Some(self.recall_utterance());
        }
        if matches!(
            lower,
            "what is my name"
                | "what's my name"
                | "do you remember my name"
                | "who am i"
                | "and my name"
                | "what about my name"
        ) {
            return Some(self.recall(Slot::Name));
        }
        if matches!(
            lower,
            "what do i prefer"
                | "what do i like"
                | "what is my preference"
                | "and my preference"
                | "what about my preference"
        ) {
            return Some(self.recall(Slot::Preference("general".into())));
        }
        if matches!(
            lower,
            "what about that" | "and that" | "what was it" | "what is it" | "and now"
        ) {
            return Some(self.last_slot.clone().map_or_else(
                || Reply::Missing {
                    text: "I don't have a name or preference in focus yet.".into(),
                },
                |s| self.recall(s),
            ));
        }
        if matches!(lower, "forget my name") {
            self.name = None;
            self.last_correction = None;
            self.last_slot = None;
            return Some(Reply::Social {
                text: "I've forgotten your name.".into(),
            });
        }
        if lower == "forget my preferences" {
            self.preferences.clear();
            self.last_correction = None;
            self.last_slot = None;
            return Some(Reply::Social {
                text: "I've forgotten your preferences.".into(),
            });
        }
        for prefix in [
            "what is my favorite ",
            "what's my favorite ",
            "what is my favourite ",
            "and my favorite ",
            "what about my favorite ",
        ] {
            if let Some(category) = after(original, prefix) {
                return Some(self.recall(Slot::Preference(category.to_lowercase())));
            }
        }
        // Strip explicit correction markers before interpreting the same typed
        // statement forms. Bare corrections require a previously focused slot.
        let mut statement = original;
        for marker in ["actually, ", "actually ", "no, ", "correction: "] {
            if let Some(rest) = after(original, marker) {
                statement = rest;
                break;
            }
        }
        for prefix in ["my name is ", "my name's "] {
            if let Some(value) = after(statement, prefix) {
                return Some(self.remember(Slot::Name, value));
            }
        }
        // "Call me a taxi" and "call me tomorrow" are not self-identifying
        // statements. The ambiguous shorthand only writes a properly cased
        // name; the explicit "my name is" form accepts any spelling/script.
        if let Some(value) = after(statement, "call me ")
            && value.split_whitespace().all(|word| {
                word.chars().next().is_some_and(char::is_uppercase)
                    && word
                        .chars()
                        .all(|c| c.is_alphabetic() || matches!(c, '-' | '\'' | '.' | '’'))
            })
        {
            return Some(self.remember(Slot::Name, value));
        }
        for prefix in ["i prefer ", "i like "] {
            if let Some(value) = after(statement, prefix) {
                return Some(self.remember(Slot::Preference("general".into()), value));
            }
        }
        for prefix in ["my favorite ", "my favourite "] {
            if let Some(rest) = after(statement, prefix) {
                // Find the separator in the original text to keep UTF-8 offsets
                // valid even when Unicode lowercase changes character width.
                let separator = rest.char_indices().find_map(|(i, _)| {
                    rest.get(i..i + 4)
                        .filter(|s| s.eq_ignore_ascii_case(" is "))
                        .map(|_| i)
                });
                if let Some(i) = separator {
                    let category = rest.get(..i).unwrap_or_default().trim().to_lowercase();
                    let value = rest.get(i + 4..).unwrap_or_default().trim();
                    if !category.is_empty() && !value.is_empty() {
                        return Some(self.remember(Slot::Preference(category), value));
                    }
                }
            }
        }
        if statement != original {
            for prefix in ["it's ", "it is ", "make that "] {
                if let Some(value) = after(statement, prefix) {
                    return Some(self.last_slot.clone().map_or_else(
                        || Reply::Missing {
                            text: "Which name or preference are you correcting?".into(),
                        },
                        |slot| self.remember(slot, value),
                    ));
                }
            }
        }
        None
    }

    fn remember(&mut self, slot: Slot, value: &str) -> Reply {
        let replacement = RememberedValue {
            value: value.to_owned(),
            stated_on_turn: self.turn,
        };
        let previous = match &slot {
            Slot::Name => self.name.replace(replacement.clone()),
            Slot::Preference(category) => self
                .preferences
                .insert(category.clone(), replacement.clone()),
        };
        self.last_correction = previous.map(|previous| Correction {
            slot: slot.clone(),
            previous,
            replacement,
        });
        self.last_slot = Some(slot.clone());
        let text = match &slot {
            Slot::Name => format!("I'll call you {value}."),
            Slot::Preference(category) if category == "general" => {
                format!("I'll remember that you prefer {value}.")
            }
            Slot::Preference(category) => {
                format!("I'll remember that your favorite {category} is {value}.")
            }
        };
        Reply::Remembered { text, slot }
    }

    fn recall(&mut self, slot: Slot) -> Reply {
        self.last_slot = Some(slot.clone());
        let (value, subject) = match &slot {
            Slot::Name => (self.name.as_ref(), "Your name".into()),
            Slot::Preference(category) if category == "general" => {
                (self.preferences.get(category), "Your preference".into())
            }
            Slot::Preference(category) => (
                self.preferences.get(category),
                format!("Your favorite {category}"),
            ),
        };
        value.map_or_else(
            || Reply::Missing {
                text: format!("I don't know {} yet.", subject.to_lowercase()),
            },
            |value| Reply::Recalled {
                text: format!("{subject} is {}.", value.value),
                source_turn: value.stated_on_turn,
            },
        )
    }

    fn recall_utterance(&self) -> Reply {
        self.last_user_utterance.as_ref().map_or_else(
            || Reply::Missing {
                text: "You haven't said anything before this turn.".into(),
            },
            |previous| Reply::Recalled {
                text: format!("You said: {previous}"),
                source_turn: self.turn.saturating_sub(1),
            },
        )
    }
}
