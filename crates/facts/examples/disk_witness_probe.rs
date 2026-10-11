//! Storage/query contract probes with prescribed synthetic relation proposals.
//! This deliberately does not measure English relation prediction or QA accuracy.
use glassmind_everyday::{Example, IntentModel, TrainOptions};
use glassmind_facts::{EntityId, disk::DiskStore};
use serde::Deserialize;
use std::{collections::BTreeSet, path::Path};

#[derive(Deserialize)]
struct Case {
    question: String,
    entity: Option<String>,
    relation: String,
    answers: BTreeSet<EntityId>,
    total: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let (database, cases, report_limits) = match args.as_slice() {
        [_, database, cases] => (database, cases, false),
        [_, database, cases, flag] if flag == "--report-set-limits" => (database, cases, true),
        _ => {
            return Err(
                "usage: disk_witness_probe DATABASE CASES.json [--report-set-limits]".into(),
            );
        }
    };
    let cases: Vec<Case> = serde_json::from_slice(&std::fs::read(cases)?)?;
    let disk = DiskStore::open(Path::new(database))?;
    let mut ordinary_answers = 0;
    let mut set_answers = 0;
    let mut refusals = 0;
    let mut truncated = 0;
    let mut largest_set = 0;
    let mut set_budget_errors = Vec::new();
    for (row, case) in cases.iter().enumerate() {
        // A two-class fixture supplies the prescribed property. It is trained
        // on the probe itself, never shipped, and is not an accuracy result.
        let mut model = IntentModel::train(
            &[
                Example {
                    text: case.question.clone(),
                    label: case.relation.clone(),
                },
                Example {
                    text: "zxqvunusedprobetoken".into(),
                    label: "unused".into(),
                },
            ],
            TrainOptions {
                epochs: 5,
                char_weight: 0.0,
                min_df: 1,
            },
        )?;
        model.policy.min_score = 0.0;
        model.policy.min_margin = 0.0;
        let expected_entity = case.entity.clone().map(EntityId);
        if disk.link(&case.question)? != expected_entity {
            return Err(format!("probe {row}: alias resolution mismatch").into());
        }
        if expected_entity.is_some() {
            let prediction = model.predict(&case.question);
            if !prediction.accepted || prediction.label != case.relation {
                return Err(format!(
                    "probe {row}: fixture did not prescribe the requested relation"
                )
                .into());
            }
        }
        let ordinary = disk.answer(&model, &case.question)?;
        if ordinary.entity != expected_entity {
            return Err(format!("probe {row}: ordinary entity mismatch").into());
        }
        if expected_entity.is_none() {
            if ordinary.refusal.is_none() || !ordinary.answers.is_empty() {
                return Err(format!("probe {row}: ambiguous alias did not refuse").into());
            }
        } else {
            if ordinary.relation.as_deref() != Some(&case.relation) {
                return Err(format!("probe {row}: ordinary relation mismatch").into());
            }
            if case.total == 1 {
                if ordinary.refusal.is_some() || ordinary.answers != case.answers {
                    return Err(format!("probe {row}: unique source answer mismatch").into());
                }
                ordinary_answers += 1;
            } else if ordinary.refusal.is_none() || !ordinary.answers.is_empty() {
                return Err(
                    format!("probe {row}: nonunique/missing source answer did not refuse").into(),
                );
            }
        }
        refusals += usize::from(ordinary.refusal.is_some());
        let set = match disk.answer_set(&model, &case.question) {
            Ok(set) => set,
            Err(error)
                if report_limits
                    && error == "interrupted"
                    && expected_entity.is_some()
                    && case.total > 10 =>
            {
                set_budget_errors.push(
                    serde_json::json!({"row":row,"expected_cardinality":case.total,"error":error}),
                );
                continue;
            }
            Err(error) => return Err(format!("probe {row}: {error}").into()),
        };
        if set.answer.entity != expected_entity
            || set.answer.answers != case.answers
            || set.total_snapshot_matches != case.total
            || set.returned != case.answers.len()
            || set.truncated != (case.total > case.answers.len())
        {
            return Err(format!("probe {row}: source-derived answer/count mismatch").into());
        }
        if expected_entity.is_none() {
            if set.answer.refusal.is_none() {
                return Err(format!("probe {row}: ambiguous set alias did not refuse").into());
            }
        } else if set.answer.relation.as_deref() != Some(&case.relation)
            || (set.answer.refusal.is_none() != (case.total > 0))
        {
            return Err(format!("probe {row}: set relation/refusal mismatch").into());
        }
        set_answers += usize::from(set.answer.refusal.is_none());
        truncated += usize::from(set.truncated);
        largest_set = largest_set.max(set.total_snapshot_matches);
    }
    if ordinary_answers == 0 || set_answers == 0 || refusals == 0 {
        return Err("fixed witness probes did not exercise successful answers and refusals".into());
    }
    println!(
        "{}",
        serde_json::json!({"cases":cases.len(),"ordinary_answers":ordinary_answers,
        "set_answers":set_answers,"ordinary_refusals":refusals,"truncated_sets":truncated,"largest_set":largest_set,
        "set_budget_errors":set_budget_errors,
        "limitation":"Synthetic prescribed relations; storage/query correctness only, not QA accuracy."})
    );
    Ok(())
}
