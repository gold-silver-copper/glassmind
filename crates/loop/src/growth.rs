//! Growth rounds: group the training failures and brief one agent per group.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use glassmind_answer::{ConversationInput, Outcome, Response, Stage};

use crate::read_jsonl;
use crate::score::{Label, Verdict};
use crate::state::Config;

/// A set of failing training turns with something in common.
#[derive(Debug, Clone)]
pub struct Group {
    /// Stable key, used to cool down groups agents fail at.
    pub key: String,
    pub title: String,
    pub turns: Vec<String>,
    pub cost: f64,
}

/// Groups the failing turns: by what was asked (task kind or conversation
/// move), and by unknown words that block many kinds of request.
pub fn groups(
    labels: &[Label],
    outcomes: &[Outcome],
    verdicts: &BTreeMap<String, (Verdict, f64)>,
) -> Vec<Group> {
    let by_id: BTreeMap<&str, &Outcome> =
        outcomes.iter().map(|o| (o.turn_id.as_str(), o)).collect();
    let mut by_kind: BTreeMap<String, Group> = BTreeMap::new();
    let mut by_word: BTreeMap<String, (Group, BTreeSet<String>)> = BTreeMap::new();
    for label in labels {
        let Some((verdict, cost)) = verdicts.get(&label.turn_id) else {
            continue;
        };
        if *verdict == Verdict::Right {
            continue;
        }
        let what = if *verdict == Verdict::Wrong {
            "answered wrongly"
        } else {
            "not handled"
        };
        let key = format!("{}:{}", label.kind, what);
        let g = by_kind.entry(key.clone()).or_insert_with(|| Group {
            key: key.clone(),
            title: format!("`{}` requests {what}", label.kind),
            turns: Vec::new(),
            cost: 0.0,
        });
        g.turns.push(label.turn_id.clone());
        g.cost += cost;
        if let Some(o) = by_id.get(label.turn_id.as_str())
            && let Response::Refuse {
                refusal: glassmind_answer::Refusal::UnknownWord { word },
            } = &o.response
        {
            let w = word.to_lowercase();
            let (g, kinds) = by_word.entry(w.clone()).or_insert_with(|| {
                (
                    Group {
                        key: format!("word:{w}"),
                        title: format!("the unknown word `{w}` across kinds of request"),
                        turns: Vec::new(),
                        cost: 0.0,
                    },
                    BTreeSet::new(),
                )
            });
            g.turns.push(label.turn_id.clone());
            g.cost += cost;
            kinds.insert(label.kind.clone());
        }
    }
    let mut out: Vec<Group> = by_kind.into_values().collect();
    out.extend(
        by_word
            .into_values()
            .filter(|(g, kinds)| kinds.len() >= 3 && g.turns.len() >= 10)
            .map(|(g, _)| g),
    );
    out.sort_by(|a, b| b.cost.total_cmp(&a.cost).then(a.key.cmp(&b.key)));
    out
}

fn short(response: &Response) -> String {
    let full = serde_json::to_string(response).unwrap_or_default();
    match response {
        Response::Items { items } if items.len() > 4 => {
            let names: Vec<&str> = items.iter().take(4).map(|i| i.name.as_str()).collect();
            format!("items ({} in all, e.g. {})", items.len(), names.join(", "))
        }
        Response::Edit { changes } => {
            let first = changes.iter().next();
            format!(
                "edit: {} changes, e.g. {}",
                changes.len(),
                first
                    .map(|c| format!(
                        "{}:{}:{} `{}` -> `{}`",
                        c.file, c.line, c.column, c.old, c.new
                    ))
                    .unwrap_or_default()
            )
        }
        _ => full.chars().take(400).collect(),
    }
}

/// Writes the brief and the data files for one group into `dir`.
pub fn write_brief(
    dir: &Path,
    config: &Config,
    round: u32,
    group: &Group,
    labels: &[Label],
    outcomes: &[Outcome],
    worked_example: Option<&str>,
) -> Result<std::path::PathBuf, String> {
    let data = dir.join("data");
    std::fs::create_dir_all(&data).map_err(|e| e.to_string())?;
    let inputs: Vec<ConversationInput> = read_jsonl(&config.train.join("inputs.jsonl"))?;
    let turn_set: BTreeSet<&str> = group.turns.iter().map(String::as_str).collect();
    let conv_of: BTreeMap<&str, &str> = labels
        .iter()
        .map(|l| (l.turn_id.as_str(), l.conversation.as_str()))
        .collect();
    let convs: BTreeSet<&str> = group
        .turns
        .iter()
        .filter_map(|t| conv_of.get(t.as_str()).copied())
        .collect();
    let write_lines = |name: &str, lines: Vec<String>| -> Result<(), String> {
        std::fs::write(data.join(name), lines.join("\n") + "\n").map_err(|e| e.to_string())
    };
    write_lines(
        "group-inputs.jsonl",
        inputs
            .iter()
            .filter(|c| convs.contains(c.id.as_str()))
            .map(line)
            .collect(),
    )?;
    write_lines(
        "group-labels.jsonl",
        labels
            .iter()
            .filter(|l| turn_set.contains(l.turn_id.as_str()))
            .map(line)
            .collect(),
    )?;
    std::fs::copy(
        config.train.join("inputs.jsonl"),
        data.join("train-inputs.jsonl"),
    )
    .map_err(|e| e.to_string())?;
    std::fs::copy(
        config.train.join("labels.jsonl"),
        data.join("train-labels.jsonl"),
    )
    .map_err(|e| e.to_string())?;
    write_lines(
        "baseline-outcomes.jsonl",
        outcomes.iter().map(line).collect(),
    )?;

    let by_turn: BTreeMap<&str, &Outcome> =
        outcomes.iter().map(|o| (o.turn_id.as_str(), o)).collect();
    let texts: BTreeMap<&str, &str> = inputs
        .iter()
        .flat_map(|c| c.turns.iter().map(|t| (t.id.as_str(), t.text.as_str())))
        .collect();
    let mut stages: BTreeMap<String, usize> = BTreeMap::new();
    let mut words: BTreeMap<String, usize> = BTreeMap::new();
    for t in &group.turns {
        if let Some(o) = by_turn.get(t.as_str()) {
            *stages.entry(format!("{:?}", o.stage)).or_default() += 1;
            if let Response::Refuse {
                refusal: glassmind_answer::Refusal::UnknownWord { word },
            } = &o.response
            {
                *words.entry(word.to_lowercase()).or_default() += 1;
            }
        }
    }
    let mut top_words: Vec<(String, usize)> = words.into_iter().collect();
    top_words.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    let mut b = String::new();
    let _ = writeln!(b, "# Loop brief: round {round}\n");
    let _ = writeln!(
        b,
        "Make glassmind handle this group of training turns: **{}** ({} turns, cost {:.1}).\n",
        group.title,
        group.turns.len(),
        group.cost
    );
    let _ = writeln!(
        b,
        "Every rule you add uses round number **{round}**: `RuleInfo::new(\"...\", {round})`.\n"
    );
    let _ = writeln!(b, "Where the group stops today: {stages:?}.");
    if !top_words.is_empty() {
        let shown: Vec<String> = top_words
            .iter()
            .take(15)
            .map(|(w, n)| format!("`{w}` ({n})"))
            .collect();
        let _ = writeln!(b, "Unknown words in this group: {}.", shown.join(", "));
    }
    let _ = writeln!(b, "\n## Examples\n");
    let mut shown = 0;
    let step = (group.turns.len() / 14).max(1);
    for t in group.turns.iter().step_by(step).take(14) {
        let Some(label) = labels.iter().find(|l| &l.turn_id == t) else {
            continue;
        };
        let conv = inputs.iter().find(|c| c.id == label.conversation);
        let earlier: Vec<&str> = conv
            .map(|c| {
                c.turns
                    .iter()
                    .take_while(|u| &u.id != t)
                    .map(|u| u.text.as_str())
                    .collect()
            })
            .unwrap_or_default();
        let _ = writeln!(b, "- turn `{t}` (source `{}`)", label.source);
        if !earlier.is_empty() {
            let _ = writeln!(b, "  - earlier in the conversation: {earlier:?}");
        }
        let _ = writeln!(
            b,
            "  - user: {:?}",
            texts.get(t.as_str()).copied().unwrap_or_default()
        );
        let _ = writeln!(b, "  - expected: {}", short(&label.expected));
        if let Some(o) = by_turn.get(t.as_str()) {
            let stage = if o.stage == Stage::Done {
                "answered".to_owned()
            } else {
                format!("{:?}", o.stage)
            };
            let _ = writeln!(b, "  - glassmind now ({stage}): {}", short(&o.response));
            let _ = writeln!(
                b,
                "  - glassmind said: {:?}",
                o.text.lines().next().unwrap_or_default()
            );
        }
        shown += 1;
    }
    if shown == 0 {
        let _ = writeln!(b, "(no examples)");
    }
    let _ = writeln!(
        b,
        "\n## Data\n\n- `SOURCES={}` (the training sources)\n- `DATA={}`: `group-inputs.jsonl` and \
         `group-labels.jsonl` (this group), `train-inputs.jsonl` and `train-labels.jsonl` (all training turns), \
         `baseline-outcomes.jsonl` (what glassmind does now).\n\nCommands to check your work are in \
         `docs/growing.md`. The exact expected answers are defined in `docs/tasks.md`.\n",
        config.sources.display(),
        data.display()
    );
    let _ = writeln!(
        b,
        "## How the change is judged\n\nThe loop accepts it only if it builds and clippy, tests and the layer \
         check pass; the training cost plus 0.01 per syntax token of mind code goes down; no training turn \
         that was right or honestly refused becomes wrong; a hidden held-out set does not get worse; and a \
         reviewer finds no keyword spotting, special-casing, or knowledge in the wrong layer. Aim for rules \
         that generalize to phrasings you have not seen: the held-out set is written by people with other \
         habits, about other repositories.\n"
    );
    if let Some(example) = worked_example {
        let _ = writeln!(
            b,
            "## An earlier accepted change, as a worked example\n\n```diff\n{example}\n```\n"
        );
    }
    let path = dir.join("brief.md");
    std::fs::write(&path, b).map_err(|e| e.to_string())?;
    Ok(path)
}

/// One JSON line.
fn line<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_default()
}
