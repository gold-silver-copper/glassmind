#![allow(clippy::panic)]
use glassmind_quantities::{Answer, solve};

#[test]
fn determined_expression_does_not_require_determined_operands() {
    for (question, expected) in [
        (
            "The sum of two numbers is 10. What is twice their sum?",
            "20",
        ),
        ("The second number is 7. What is the second number?", "7"),
        (
            "A number is the number. What is the number minus the number?",
            "0",
        ),
    ] {
        match solve(question) {
            Answer::Answer { value, unit, .. } => {
                assert_eq!(value, expected);
                assert_eq!(unit, None);
            }
            other => panic!("{question}: {other:?}"),
        }
    }
}

#[test]
fn sign_constraints_can_determine_the_target() {
    for (question, expected) in [
        (
            "The difference between the first number and the second number is 3. What is the difference between the first number and the second number?",
            "3",
        ),
        (
            "The sum of the difference between the first number and 0 and the difference between the second number and 0 is 0. What is their sum?",
            "0",
        ),
    ] {
        match solve(question) {
            Answer::Answer { value, unit, .. } => {
                assert_eq!(value, expected);
                assert_eq!(unit, None);
            }
            other => panic!("{question}: {other:?}"),
        }
    }
}

#[test]
fn free_inputs_and_inconsistent_branches_do_not_supply_answers() {
    for question in [
        "The sum of two numbers is 10. What is the first number?",
        "The difference between the first number and the second number is 3. What is the first number minus the second number?",
        "The sum of two numbers is 10. Twice their sum is 21. What is their sum?",
        "The sum of the difference between the first number and 0 and the difference between the second number and 0 is 1. What is their sum?",
    ] {
        assert!(
            matches!(solve(question), Answer::Refusal { .. }),
            "{question}"
        );
    }
}
