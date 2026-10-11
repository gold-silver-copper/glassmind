//! Public authored development checks for proposal scope and legacy priority.
use glassmind_world_rust::checks;

#[test]
fn multiplicative_proposals_are_expression_tokens_and_legacy_order_is_global() {
    let root = checks::scratch("multiplicative-proposals");
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("tests")).unwrap();
    let sources = [
        (
            "src/lib.rs",
            "mod early; mod late; #[cfg(test)] mod helper;\nfn scale(café:i32)->i32 { café * 4 }\nfn legacy(x:i32)->i32 { x + 1 }\n",
        ),
        (
            "src/early.rs",
            r#"// a * b / c
const TEXT: &str = "* /";
fn f(café:i32)->i32 { café / 4 }
fn pointers(x: *const i32, r: &i32) { let _ = *r; }
fn assignment(mut x:i32) { x *= 4; x /= 4; }
fn macros(x:i32) { assert_eq!(x * 4, x / 4); }
#[test] fn test() { let _ = 8 * 4; }
#[cfg(test)] const EXPECTED: i32 = 8 / 4;
fn conditional(x:i32) { #[cfg(test)] let _ = x * 4; #[cfg(test)] { let _ = x / 4; } }
"#,
        ),
        ("src/late.rs", "fn legacy(x:i32)->i32 { x - 1 }\n"),
        ("src/helper.rs", "fn helper(x:i32)->i32 { x * 4 }\n"),
        ("tests/visible.rs", "fn visible(x:i32)->i32 { x / 4 }\n"),
    ];
    for (file, text) in sources {
        std::fs::write(root.join(file), text).unwrap();
    }
    let changes = checks::slips("probe", &root, &[]);
    let actual: Vec<_> = changes
        .iter()
        .map(|c| (c.file.as_str(), c.old.as_str(), c.new.as_str()))
        .collect();
    assert_eq!(
        actual,
        [
            ("src/late.rs", "-", "+"),
            ("src/lib.rs", "+", "-"),
            ("src/early.rs", "/", "*"),
            ("src/lib.rs", "*", "/"),
        ]
    );
    // Every span uses character columns even after a multi-byte identifier.
    for change in changes {
        let text = std::fs::read_to_string(root.join(&change.file)).unwrap();
        let token: String = text
            .lines()
            .nth(change.line as usize - 1)
            .unwrap()
            .chars()
            .skip(change.column as usize)
            .take(change.old.chars().count())
            .collect();
        assert_eq!(token, change.old);
    }
    std::fs::remove_dir_all(root).unwrap();
}
