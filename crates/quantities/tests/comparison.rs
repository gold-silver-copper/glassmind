use glassmind_quantities::{
    comparison::*,
    rational::Rational,
    units::{Dimension, Quantity, Temperature, TypedUnit},
};
use serde_json::Value;
use std::cmp::Ordering;

#[test]
fn frozen_authored_contract() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<Value> = serde_json::from_str(include_str!(
        "../../../tools/quantities/unit-comparison/authored.json"
    ))?;
    for case in cases {
        let question = case["question"].as_str().ok_or("missing question")?;
        let response = serde_json::to_value(ComparisonResponse::from(compare_question(question)))?;
        if case.get("refusal").is_some() {
            assert_eq!(response["status"], "refusal", "{question}");
            assert_eq!(response["error"]["kind"], case["refusal"], "{question}");
        } else {
            assert_eq!(response["status"], "comparison", "{question}: {response}");
            let result = &response["result"];
            assert_eq!(result["dimension"], case["dimension"], "{question}");
            assert_eq!(result["ordering"], case["ordering"], "{question}");
            assert_eq!(
                result["witness"]["left"], case["witness_left"],
                "{question}"
            );
            assert_eq!(
                result["witness"]["right"], case["witness_right"],
                "{question}"
            );
            assert_eq!(result["outcome"]["kind"], case["kind"], "{question}");
            let (op, answer) = if case["kind"] == "predicate" {
                ("predicate", "truth")
            } else {
                ("selection", "selected")
            };
            assert_eq!(result["outcome"][op], case["operation"], "{question}");
            assert_eq!(result["outcome"][answer], case["answer"], "{question}");
        }
    }
    Ok(())
}
fn q(value: &str, unit: &str) -> Result<ComparisonQuantity, Box<dyn std::error::Error>> {
    Ok(ComparisonQuantity::new(value.parse::<Rational>()?, unit)?)
}
#[test]
fn direct_queries_validate_and_match_parser() -> Result<(), Box<dyn std::error::Error>> {
    let direct = ComparisonQuery {
        left: q("-1/3", "hours")?,
        right: q("-20", "minutes")?,
        operation: ComparisonOperation::Predicate(Predicate::EqualTo),
    };
    assert_eq!(
        serde_json::to_value(direct.evaluate()?)?,
        serde_json::to_value(compare_question("Is -1/3 hours equal to -20 minutes?")?)?
    );
    let wrong_dimension = ComparisonQuery {
        left: q("1", "kg")?,
        right: q("1", "m")?,
        operation: ComparisonOperation::Predicate(Predicate::GreaterThan),
    };
    assert!(matches!(
        wrong_dimension.evaluate(),
        Err(ComparisonError::DimensionMismatch)
    ));
    let wrong_selection = ComparisonQuery {
        left: q("1", "kg")?,
        right: q("2", "kg")?,
        operation: ComparisonOperation::Selection(Selection::Longer),
    };
    assert!(matches!(
        wrong_selection.evaluate(),
        Err(ComparisonError::SelectionDimensionMismatch)
    ));
    assert!(ComparisonQuantity::new(Rational::integer(1), "gallons").is_err());
    assert!(ComparisonQuantity::new(Rational::integer(1), "m\0").is_err());
    assert!(ComparisonQuantity::new(Rational::integer(1), &"m".repeat(4097)).is_err());
    Ok(())
}
#[test]
fn every_predicate_all_orders_and_inverse_facts() -> Result<(), Box<dyn std::error::Error>> {
    for (left_unit, right_unit, ratio, dimension) in [
        ("m", "cm", 100, Dimension::Length),
        ("kg", "g", 1000, Dimension::Mass),
        ("hours", "minutes", 60, Dimension::Duration),
        ("l", "ml", 1000, Dimension::Volume),
        ("m/s", "m/s", 1, Dimension::Speed),
        ("kelvin", "kelvin", 1, Dimension::Temperature),
    ] {
        for right_value in [ratio - 1, ratio, ratio + 1] {
            let ordering = ratio.cmp(&right_value);
            for (predicate, truth) in [
                (Predicate::GreaterThan, ordering.is_gt()),
                (Predicate::LessThan, ordering.is_lt()),
                (Predicate::EqualTo, ordering.is_eq()),
                (Predicate::AtLeast, !ordering.is_lt()),
                (Predicate::AtMost, !ordering.is_gt()),
            ] {
                let query = ComparisonQuery {
                    left: q("1", left_unit)?,
                    right: q(&right_value.to_string(), right_unit)?,
                    operation: ComparisonOperation::Predicate(predicate),
                };
                let result = query.evaluate()?;
                assert_eq!(result.dimension, dimension);
                assert_eq!(result.ordering, ordering.into());
                assert_eq!(
                    result.outcome,
                    ComparisonOutcome::Predicate { predicate, truth }
                );
                let inverse = ComparisonQuery {
                    left: query.right,
                    right: query.left,
                    operation: query.operation,
                }
                .evaluate()?;
                assert_eq!(inverse.ordering, ordering.reverse().into());
            }
        }
    }
    Ok(())
}
#[test]
fn every_selection_all_orders_including_explicit_equality() -> Result<(), Box<dyn std::error::Error>>
{
    for (selection, unit, greater) in [
        (Selection::Longer, "m", true),
        (Selection::Shorter, "m", false),
        (Selection::Heavier, "kg", true),
        (Selection::Lighter, "kg", false),
        (Selection::Faster, "m/s", true),
        (Selection::Slower, "m/s", false),
        (Selection::Hotter, "celsius", true),
        (Selection::Colder, "celsius", false),
    ] {
        for right in [0, 1, 2] {
            let result = ComparisonQuery {
                left: q("1", unit)?,
                right: q(&right.to_string(), unit)?,
                operation: ComparisonOperation::Selection(selection),
            }
            .evaluate()?;
            let selected = if right == 1 {
                SelectedOperand::Equal
            } else if (right < 1) == greater {
                SelectedOperand::Left
            } else {
                SelectedOperand::Right
            };
            assert_eq!(
                result.outcome,
                ComparisonOutcome::Selection {
                    selection,
                    selected
                }
            );
        }
    }
    Ok(())
}
#[test]
fn rational_ordering_overflow_and_static_temperature() -> Result<(), Box<dyn std::error::Error>> {
    let max = Rational::new(i128::MAX, 1)?;
    assert_eq!(max.checked_cmp(max)?, Ordering::Equal);
    assert_eq!(max.checked_cmp(max.checked_neg()?)?, Ordering::Greater);
    assert!(max.checked_cmp(Rational::new(1, i128::MAX)?).is_err());
    assert!(Rational::new(1, 0).is_err());
    let freezing_c =
        Quantity::<Temperature>::new(Rational::integer(0), TypedUnit::parse("celsius")?);
    let freezing_f =
        Quantity::<Temperature>::new(Rational::integer(32), TypedUnit::parse("fahrenheit")?);
    assert_eq!(freezing_c.checked_cmp(&freezing_f)?, Ordering::Equal);
    for a in -8..=8 {
        for b in 1..=8 {
            for c in -8..=8 {
                let x = Rational::new(a, b)?;
                let y = Rational::new(c, 3)?;
                assert_eq!(x.checked_cmp(y)?, (a * 3).cmp(&(c * b)));
            }
        }
    }
    Ok(())
}
#[test]
fn full_consumption_boundaries_and_punctuation_spacing() {
    let base = "Which is longer,1 m or 2 m?";
    assert!(compare_question(base).is_ok());
    assert!(compare_question("Which is longer , 1 m or 2 m?").is_ok());
    let exact = format!(
        "{}Is 1 m equal to 1 m?",
        " ".repeat(4096 - "Is 1 m equal to 1 m?".len())
    );
    assert!(compare_question(&exact).is_ok());
    assert!(matches!(
        compare_question(&(exact + " ")),
        Err(ComparisonError::InvalidInput)
    ));
    for extra in [
        " remember Ada",
        "? another question",
        " or 3 m",
        "; set x = 1",
        "\0",
    ] {
        assert!(compare_question(&format!("Is 1 m equal to 1 m{extra}")).is_err());
    }
}
