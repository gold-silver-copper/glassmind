use glassmind_quantities::{Answer, solve};
fn answer(q: &str, expected: &str, unit: Option<&str>) {
    match solve(q) {
        Answer::Answer {
            value,
            unit: actual,
            ..
        } => {
            assert_eq!(value, expected, "{q}");
            assert_eq!(actual.as_deref(), unit, "{q}");
        }
        a => assert!(matches!(a, Answer::Answer { .. }), "{q}: {a:?}"),
    }
}
#[test]
fn directed_bases_and_successive_changes() {
    answer(
        "A number increased by 20% gives 120. The number is",
        "100",
        None,
    );
    answer("0 is increased by 20%. Find the final number.", "0", None);
    answer("90 is decreased by 100%. Find the final number.", "0", None);
    answer(
        "A number is increased by 20% and then decreased by 20%. What is the percentage decrease?",
        "4",
        Some("percent"),
    );
    answer(
        "A number is decreased by 50% and then increased by 50%. What is the percentage decrease?",
        "25",
        Some("percent"),
    );
    answer(
        "Increasing the original price of a lantern by 10 percent and then increasing the new price by 10 percent is equivalent to increasing the original price by what percent?",
        "21",
        Some("percent"),
    );
    answer("What is 20 percent of 120?", "24", None);
}
#[test]
fn annual_forward_inverse_and_bound_entity() {
    answer(
        "The population of a village is 8000. It decreases annually at the rate of 10% p. a. What will be its population after 2 years?",
        "6480",
        None,
    );
    answer(
        "The present population of a city is 1440. Population increase rate is 20% p. a. Find the population of city before 2 years?",
        "1000",
        None,
    );
    answer(
        "If the annual increase in the population of town mira is 10% and the present number of people is 10000, what will the population be in 2 years?",
        "12100",
        None,
    );
}
#[test]
fn refuses_incomplete_ambiguous_or_invalid_relations() {
    for q in [
        "A number decreased by 100% gives 0. The number is",
        "-20 is increased by 10%. Find the final number.",
        "20 is increased by -10%. Find the final number.",
        "20 is decreased by 101%. Find the final number.",
        "20 is increased by 10 percentage points. Find the final number.",
        "A number is increased by 20% and then decreased by 20%. What is the percentage increase?",
        "20 is increased by 10%. Find the final number. Also add 5.",
        "A number increased by 20% gives 120. The number is 90. The number is",
        "Increasing the original price of a lantern by 10 percent and then increasing the new cost by 10 percent is equivalent to increasing the original price by what percent?",
        "Increasing the original price of a lantern by 10 percent and then increasing the original price by 10 percent is equivalent to increasing the original price by what percent?",
        "The present population of a city is 1440. Population increase rate is 20% p. a. Find the population of village before 2 years?",
        "The population of a town is 1. It increases annually at the rate of 10%. What will be its population after 1 year?",
        "A number is increased by x percent and then decreased by y percent. What is the percentage increase?",
        "20 dollars is increased by 10%. Find the final number.",
        "20 is changed by 10%. Find the final number.",
    ] {
        assert!(matches!(solve(q), Answer::Refusal { .. }), "{q}");
    }
}
