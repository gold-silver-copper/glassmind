//! Authored structural checks; end-to-end native fixtures live in tools/coding-ceiling-division.
use glassmind_world_rust::checks;

#[test]
fn rounding_is_after_every_legacy_proposal_across_files() {
    let root = checks::scratch("rounding-order");
    std::fs::create_dir_all(root.join("src")).unwrap();
    let files = [
        (
            "src/lib.rs",
            "mod early; mod late;\nfn q(n:u32)->u32 { n / 3 }\nfn old(n:u32)->u32 { n + 1 }\n",
        ),
        ("src/early.rs", "fn scale(n:u32)->u32 { n * 3 }\n"),
        ("src/late.rs", "fn old(n:u32)->u32 { n - 1 }\n"),
    ];
    for (file, text) in files {
        std::fs::write(root.join(file), text).unwrap();
    }
    let changes = checks::slips("s", &root, &[]);
    let actual: Vec<_> = changes
        .iter()
        .map(|c| (c.file.as_str(), c.old.as_str(), c.new.as_str()))
        .collect();
    assert_eq!(
        actual,
        [
            ("src/late.rs", "-", "+"),
            ("src/lib.rs", "+", "-"),
            ("src/early.rs", "*", "/"),
            ("src/lib.rs", "/", "*"),
            (
                "src/lib.rs",
                "n / 3",
                "::core::primitive::u32::div_ceil(n, 3)"
            ),
        ]
    );
    for (file, text) in files {
        assert_eq!(std::fs::read_to_string(root.join(file)).unwrap(), text);
    }
    std::fs::remove_dir_all(root).unwrap();
}
