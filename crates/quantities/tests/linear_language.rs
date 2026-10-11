use glassmind_quantities::{Answer, solve};

fn answer(q: &str, expected: &str) {
    match solve(q) {
        Answer::Answer { value, unit, .. } => {
            assert_eq!(value, expected, "{q}");
            assert_eq!(unit, None);
        }
        other => assert!(matches!(other, Answer::Answer { .. }), "{q}: {other:?}"),
    }
}

#[test]
fn training_concepts_use_shared_scalar_constraints() {
    answer(
        "A number minus three-fifths of the number is 50. What is the number?",
        "125",
    );
    answer(
        "65 % of a number is 21 less than 4 / 5 th of that number. What is the number?",
        "140",
    );
    answer(
        "50 % of a number is more than 20 % of 650 by 190. Find the number.",
        "640",
    );
    answer(
        "3 seventh of a number is 12 more than 40 % of that number. What is 30 % of that number?",
        "126",
    );
}

#[test]
fn absolute_differences_require_agreement_across_solutions() {
    answer(
        "The first number is 2. The second number is 3. What is the difference between the first number and the second number?",
        "1",
    );
    answer(
        "The first number is -2. The second number is 3. What is twice the difference between the first number and the second number?",
        "10",
    );
    answer(
        "A number is 7. The difference between the number and 10 is 3. Find the number.",
        "7",
    );
    answer(
        "The difference between a number and 0 is 5. What is the difference between the number and 0?",
        "5",
    );
    for question in [
        "The difference between a number and 10 is 3. Find the number.",
        "The difference between a number and its three-fifths is 50. Find the number.",
        "A number is 7. The difference between the number and 10 is -3. Find the number.",
    ] {
        assert!(
            matches!(solve(question), Answer::Refusal { .. }),
            "{question}"
        );
    }
}

#[test]
fn novel_clause_combinations_and_order() {
    answer(
        "Twice a number plus 7 is 31. What is 25 percent of the number?",
        "3",
    );
    answer(
        "The sum of two numbers is 31 and the first number is 3 more than the second number. Find twice the second number.",
        "28",
    );
    answer(
        "The first number is 3 more than the second number; their sum is 31. Find twice the second number.",
        "28",
    );
    answer(
        "The second number is 2 times the first number. Their sum is 15. The first number is 5. What is the difference between the second number and the first number?",
        "5",
    );
    answer(
        "A number is 7 less than twice the number. Find the number.",
        "7",
    );
    answer(
        "The sum of a number and twice the number is 7. Find the number.",
        "7/3",
    );
    answer(
        "The first number is 2. The second number is 3. What is their sum?",
        "5",
    );
    answer("A number is -3. What is twice the number?", "-6");
}

#[test]
fn omissions_contradictions_units_and_ambiguous_referents_refuse() {
    for q in [
        "The sum of two numbers is 31. Find the first number.",
        "The sum of two numbers is 31. Their sum is 32. Find the first number.",
        "A number is 7. The number is 8. Find the number.",
        "Twice a number is twice the number. Find the number.",
        "Twice a number is twice the number plus 1. Find the number.",
        "Twice a number is 8 kilograms. Find the number.",
        "Twice a number is 8. Find the number. It is even.",
        "Twice a number is 8. Find the number in meters.",
        "Twice a number is 8. What is the other number?",
        "A number is 8 and the first number is 2. Find the number.",
        "A number is 8 and another number is 2. Find the number.",
        "The sum of two numbers is 31 and one is 3 more than the other. Find the number.",
        "The number is 8. Find the number.",
        "A number is 8. A number is 8. Find the number.",
        "A number is more than 8. Find the number.",
        "A number is 8. Find the number rounded to tens.",
        "A number is 8. Find the number times the number.",
        "0 times a number is 0. Find the number.",
        "1/0 of a number is 7. Find the number.",
        "A number is 99999999999999999999999999999999999999999999. Find the number.",
    ] {
        assert!(matches!(solve(q), Answer::Refusal { .. }), "{q}");
    }
}
