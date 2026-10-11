use glassmind_quantities::{Answer, solve};

fn answer(question: &str, expected: &str) {
    let actual = solve(question);
    assert!(
        matches!(actual, Answer::Answer { .. }),
        "refused {question}"
    );
    if let Answer::Answer { value, unit, .. } = actual {
        assert_eq!(value, expected, "{question}");
        assert_eq!(unit, None);
    }
}

#[test]
fn train_problem_only_examples_with_independent_arithmetic() {
    for (q, value) in [
        (
            "the ratio of three numbers is 2 : 3 : 5 and their sum is 150 . the second number of the three numbers is ?",
            "45",
        ),
        (
            "four numbers are in the ratio 2 : 3 : 4 : 5 add up to give a sum of 1344 . find the biggest number .",
            "480",
        ),
        (
            "the ratio of two numbers is 3 : 4 and their sum is 21 . the greater of the two numbers is ?",
            "12",
        ),
        (
            "the ratio of 3 numbers is 5 : 1 : 4 and their sum is 1000 . the last number of the 3 numbers is ?",
            "400",
        ),
        (
            "the sum of 3 numbers is 98 . the ratio of the first to the second is 2 / 3 , and the ratio of the second to the third is 5 / 8 . the second number is :",
            "30",
        ),
        (
            "the sum of 3 numbers is 98 . if the ratio between first and second be 2 : 3 and that between second and third be 5 : 8 , then the second number is ?",
            "30",
        ),
        (
            "the sum of 3 numbers is 98 . if the ratio of the first to the second is 2 : 3 and that of the second to the third is 5 : 8 , then the second number is :",
            "30",
        ),
        (
            "the sum of the numbers is 177 . if the ratio between the first and the second be 5 : 3 and that between the second and third be 4 : 9 , then find the second number ?",
            "36",
        ),
    ] {
        answer(q, value);
    }
}

#[test]
fn independent_exact_cases_bind_order_and_rank() {
    answer(
        "The ratio of two numbers is 7:2 and their sum is 20. The first number is?",
        "140/9",
    );
    answer(
        "The ratio of four numbers is 9:2:7:1 and their sum is 38. Find the smallest number.",
        "2",
    );
    answer(
        "Three numbers are in the ratio 0.5 to 1.5 to 3. Their sum is 25. Find the last number.",
        "15",
    );
    answer(
        "The sum of three numbers is 26. The ratio of the first to the second is 3:2 and that of the second to the third is 4:3. The third number is?",
        "6",
    );
}

#[test]
fn unsupported_or_inconsistent_constraints_refuse() {
    for q in [
        "The sum of three numbers is 26. The ratio of the first to the second is 2/3/4 and that of the second to the third is 4:3. The third number is?",
        "The ratio of three numbers is 1:2:3 and their sum is 60. The greater of the three numbers is?",
        "The ratio of two numbers is 1:2:3 and their sum is 60. Find the first number.",
        "The ratio of three numbers is 1:2:3 and their sum is 60. The first number of the two numbers is?",
        "The ratio of two numbers is 1:2 and their sum is 60. Find the third number.",
        "The sum of four numbers is 26. The ratio of the first to the second is 3:2 and that of the second to the third is 4:3. The third number is?",
        "The ratio of two numbers is 0:2 and their sum is 60. Find the first number.",
        "The ratio of two numbers is 1/0:2 and their sum is 60. Find the first number.",
        "The ratio of two numbers is 1:2 and their sum is 0. Find the first number.",
        "The ratio of two numbers is 1:2 and their sum is -60. Find the first number.",
        "The ratio of two whole numbers is 1:2 and their sum is 20. Find the first number.",
        "The ratio of two numbers is 1:2 and their sum is 60 kilograms. Find the first number.",
        "The ratio of two numbers is 1:2 and their sum is 60. Find the first number rounded to tens.",
        "The ratio of two numbers is 1:2 and their sum is 60. Find the first number. It is even.",
        "The ratio of two numbers is 1:2 and their sum is 60. Find the difference.",
        "The ratio of two numbers is 1:2 and their sum of squares is 60. Find the first number.",
        "The ratio of two numbers is 9999999999999999999999999999999999999999999999:2 and their sum is 60. Find the first number.",
        "The ratio of two numbers is 100000000000000000000:2 and their sum is 100000000000000000000. Find the first number.",
    ] {
        assert!(matches!(solve(q), Answer::Refusal { .. }), "{q}");
    }
}
