use glassmind_quantities::{Answer, solve};

#[test]
fn percentage_relations_and_composition() {
    for (q, expected, unit) in [
        ("What is 12.5% of 3 / 7 of 56?", "3", None),
        ("Calculate 20 percent of 25% of 160", "8", None),
        ("What is 0.5 percent of 14,200?", "71", None),
        ("What percentage of 80 is 12?", "15", Some("percent")),
        ("18 is what percent of 72?", "25", Some("percent")),
        (
            "What percent of 12 is 8 percent of 6?",
            "4",
            Some("percent"),
        ),
        ("What percent is 9 of 24?", "37.5", Some("percent")),
        ("24 % of 50 is", "12", None),
    ] {
        let Answer::Answer {
            value,
            unit: actual_unit,
            ..
        } = solve(q)
        else {
            panic!("refused {q}");
        };
        // Rational rendering may use a fraction for a non-integer answer.
        assert_eq!(
            value
                .parse::<glassmind_quantities::rational::Rational>()
                .unwrap(),
            expected
                .parse::<glassmind_quantities::rational::Rational>()
                .unwrap(),
            "{q}"
        );
        assert_eq!(actual_unit.as_deref(), unit, "{q}");
    }
}

#[test]
fn malformed_or_unconsumed_percentage_relations_refuse() {
    for q in [
        "What percent of 0 is 9?",
        "What is 15% of 2/0?",
        "What is 10% of 1,20?",
        "What is 10% of 1, 200?",
        "What is 10% of 5 apples and 9 pears?",
        "What percent of 50 is 10 except for tax?",
        "What is 10% of 2 / 3 / 4?",
        "What is 10% of 5 or 9?",
        "What is 99999999999999999999999999999999999999999999% of 4?",
    ] {
        assert!(matches!(solve(q), Answer::Refusal { .. }), "{q}");
    }
}
