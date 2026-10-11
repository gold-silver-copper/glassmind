//! Authored public development fixtures for diagnostic isolation, not a benchmark.
#[cfg(test)]
mod tests {
    use glassmind_meaning::{Action, Reply, interface::World};
    use glassmind_rules::Switches;
    use glassmind_world_rust::{
        RustKind, RustWorld,
        checks::{self, Places, RustChange, RustCheck},
    };
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};

    const MANIFEST: &str =
        "[package]\nname=\"isolation_probe\"\nversion=\"0.0.0\"\nedition=\"2024\"\n[workspace]\n";
    const LOCK: &str =
        "version = 4\n[[package]]\nname = \"isolation_probe\"\nversion = \"0.0.0\"\n";
    const PASS: &str = "pub fn x() -> u8 { 7 }\n#[test] fn works() { assert_eq!(x(), 7); }\n";
    const FAIL: &str = "pub fn x() -> u8 { 7 }\n#[test] fn broken() { assert_eq!(x(), 8); }\n";
    const TODO: &str =
        "pub fn x() -> u8 { todo!() }\n#[test] fn unfinished() { assert_eq!(x(), 8); }\n";
    const FIX: &str =
        "pub fn x() -> u8 { let n = 7; n += 1; n }\n#[test] fn works() { assert_eq!(x(), 8); }\n";

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = checks::scratch("isolation-fixture");
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn package(&self, name: &str, code: &str, lock: bool) -> PathBuf {
            let root = self.0.join(name).join("package");
            std::fs::create_dir_all(root.join("src")).unwrap();
            std::fs::create_dir_all(root.join("target")).unwrap();
            std::fs::create_dir_all(root.join(".kept")).unwrap();
            std::fs::write(root.join("Cargo.toml"), MANIFEST).unwrap();
            std::fs::write(root.join("src/lib.rs"), code).unwrap();
            std::fs::write(root.join("target/keep"), b"existing build\0").unwrap();
            std::fs::write(root.join(".kept/keep"), b"hidden\0").unwrap();
            if lock {
                std::fs::write(root.join("Cargo.lock"), LOCK).unwrap();
            }
            root
        }
        fn places(&self, owned: bool) -> Places {
            if owned {
                Places {
                    target: Some(
                        PathBuf::from(std::env::var_os("CARGO_TARGET_DIR").unwrap())
                            .join(self.0.file_name().unwrap()),
                    ),
                    cache: Some(self.0.join("results")),
                }
            } else {
                Places::default()
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    // Unlike production copy enumeration, includes target, hidden files and dirs.
    fn snapshot(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        fn walk(root: &Path, dir: &Path, all: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                let kind = entry.file_type().unwrap();
                let bytes = if kind.is_symlink() {
                    format!("link:{}", std::fs::read_link(&path).unwrap().display()).into_bytes()
                } else if kind.is_dir() {
                    walk(root, &path, all);
                    b"directory".to_vec()
                } else {
                    std::fs::read(&path).unwrap()
                };
                all.insert(path.strip_prefix(root).unwrap().to_owned(), bytes);
            }
        }
        let mut all = BTreeMap::new();
        walk(root, root, &mut all);
        all
    }

    #[test]
    fn diagnostics_preserve_every_live_byte_with_default_and_owned_places() {
        let fixture = Fixture::new();
        for owned in [false, true] {
            let places = fixture.places(owned);
            for lock in [false, true] {
                for (name, code, failure) in [
                    ("pass", PASS, None),
                    ("fail", FAIL, Some("broken")),
                    ("todo", TODO, Some("unfinished")),
                    ("compiler", FIX, Some("E0384")),
                ] {
                    let root = fixture.package(&format!("{name}-{owned}-{lock}"), code, lock);
                    let before = snapshot(&root);
                    let findings =
                        checks::run_in(&places, RustCheck::Test, "package", &root).unwrap();
                    assert_eq!(findings.is_empty(), failure.is_none(), "{name}");
                    if let Some(name) = failure {
                        assert!(findings.iter().any(|f| f.item.name == name), "{findings:?}");
                    }
                    assert_eq!(snapshot(&root), before);
                    let again = checks::run_in(&places, RustCheck::Test, "package", &root).unwrap();
                    if owned {
                        // An actual result-cache hit preserves even the raw detail.
                        assert_eq!(again, findings);
                    } else {
                        // Fresh panic messages contain OS-assigned thread IDs.
                        let stable = |mut found: Vec<checks::Finding>| {
                            for finding in &mut found {
                                if let Some(detail) = &mut finding.item.detail
                                    && let Some((prefix, _)) = detail.split_once(") panicked")
                                    && let Some((_, id)) = prefix.rsplit_once('(')
                                    && id.chars().all(|c| c.is_ascii_digit())
                                {
                                    *detail = detail.replacen(
                                        &format!("({id}) panicked"),
                                        "(PID) panicked",
                                        1,
                                    );
                                }
                            }
                            found
                        };
                        assert_eq!(stable(again), stable(findings));
                    }
                    assert_eq!(snapshot(&root), before);
                }
            }
        }
    }

    #[test]
    fn same_name_shared_target_and_cache_never_reuse_another_sources_binary() {
        let fixture = Fixture::new();
        let places = fixture.places(true);
        let a = fixture.package("a", PASS, false);
        let b = fixture.package("b", FAIL, false);
        for root in [&a, &b, &a, &b] {
            let found = checks::run_in(&places, RustCheck::Test, "package", root).unwrap();
            assert_eq!(found.is_empty(), root == &a);
        }
        // Change the same path's bytes while preserving its original mtime.
        // The copied future mtime must still invalidate the shared Cargo result.
        let path = a.join("src/lib.rs");
        let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
        std::fs::write(&path, TODO).unwrap();
        std::fs::File::open(&path)
            .unwrap()
            .set_modified(mtime)
            .unwrap();
        let before = snapshot(&a);
        let found = checks::run_in(&places, RustCheck::Test, "package", &a).unwrap();
        assert!(found.iter().any(|f| f.item.name == "unfinished"));
        assert_eq!(snapshot(&a), before);
        let warnings = fixture.package("lint", "pub fn x() { let unused = 7; }\n", false);
        let before = snapshot(&warnings);
        assert!(
            checks::run_in(&places, RustCheck::Lint, "other-source", &warnings)
                .unwrap()
                .iter()
                .any(|f| f.item.kind == RustKind::Problem)
        );
        assert_eq!(snapshot(&warnings), before);
    }

    #[test]
    fn approved_repair_stale_rules_and_failed_verification_preserve_contract() {
        let fixture = Fixture::new();
        let root = fixture.package("repair", FIX, false);
        let world =
            |switches: &Switches| RustWorld::open(&[("package".into(), root.as_path())], switches);
        let before = snapshot(&root);
        let world_on = world(&Switches::all_on());
        let proposal = world_on
            .propose(&Action::Repair(RustCheck::Build), &mut vec![])
            .unwrap();
        assert!(
            matches!(&proposal, Reply::Remedy { changes, left, .. } if changes.len() == 1 && left.is_empty())
        );
        assert_eq!(snapshot(&root), before);
        assert!(
            world(&Switches::without(["rust.apply.changes".to_owned()]))
                .apply(&proposal, &mut vec![])
                .is_err()
        );
        assert_eq!(snapshot(&root), before);
        assert!(matches!(
            world_on.apply(&proposal, &mut vec![]).unwrap(),
            Reply::Applied { changes: 1, .. }
        ));
        let mut expected = before;
        expected.insert(
            PathBuf::from("src/lib.rs"),
            FIX.replace("let n =", "let mut n =").into_bytes(),
        );
        assert_eq!(snapshot(&root), expected);
        assert!(world_on.apply(&proposal, &mut vec![]).is_err());
        assert_eq!(snapshot(&root), expected);
        // A type-correct but test-breaking edit must roll back all source bytes.
        std::fs::write(root.join("src/lib.rs"), PASS).unwrap();
        let before = snapshot(&root);
        let bad = Reply::Remedy {
            check: RustCheck::Test,
            changes: vec![RustChange {
                source: "package".into(),
                file: "src/lib.rs".into(),
                line: 1,
                column: 19,
                old: "7".into(),
                new: "8".into(),
            }],
            fixed: vec![],
            left: vec![],
        };
        assert!(world(&Switches::all_on()).apply(&bad, &mut vec![]).is_err());
        assert_eq!(snapshot(&root), before);
    }

    #[test]
    fn copy_rules_scripts_and_unsupported_topology_do_not_fall_back_to_live() {
        let fixture = Fixture::new();
        let root = fixture.package("script", PASS, false);
        std::fs::write(
            root.join("build.rs"),
            "fn main() { std::fs::write(\"build-marker\", b\"scratch only\").unwrap(); }\n",
        )
        .unwrap();
        let linked = fixture.0.join("linked.rs");
        std::fs::write(&linked, PASS).unwrap();
        std::fs::remove_file(root.join("src/lib.rs")).unwrap();
        std::os::unix::fs::symlink(&linked, root.join("src/lib.rs")).unwrap();
        let before = snapshot(&root);
        assert!(
            checks::run_in(&Places::default(), RustCheck::Test, "package", &root)
                .unwrap()
                .is_empty()
        );
        assert_eq!(snapshot(&root), before);
        assert_eq!(std::fs::read_to_string(&linked).unwrap(), PASS);
        // Existing copy rules materialize file links, but directory links cannot
        // be copied as files and must fail without trying cargo at the live root.
        std::os::unix::fs::symlink(root.join("src"), root.join("linked-dir")).unwrap();
        let before = snapshot(&root);
        assert!(checks::run_in(&Places::default(), RustCheck::Build, "package", &root).is_err());
        assert_eq!(snapshot(&root), before);
        let relative = fixture.package("relative", PASS, false);
        let outside = relative.parent().unwrap().join("outside");
        std::fs::create_dir_all(outside.join("src")).unwrap();
        std::fs::write(
            outside.join("Cargo.toml"),
            MANIFEST.replace("isolation_probe", "missing"),
        )
        .unwrap();
        std::fs::write(outside.join("src/lib.rs"), "pub fn outside() {}\n").unwrap();
        std::fs::write(
            relative.join("Cargo.toml"),
            format!("{MANIFEST}\n[dependencies]\nmissing={{path=\"../outside\"}}\n"),
        )
        .unwrap();
        let before = snapshot(&relative);
        assert!(
            checks::run_in(&Places::default(), RustCheck::Build, "package", &relative).is_err()
        );
        assert_eq!(snapshot(&relative), before);
    }
}
