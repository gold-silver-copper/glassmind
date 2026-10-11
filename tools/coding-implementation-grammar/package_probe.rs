use glassmind_world_rust::{RustWorld, implementation::Implement};
use glassmind_rules::Switches;
use std::{fs, path::PathBuf};
fn main() {
    let base = PathBuf::from(std::env::args().nth(1).unwrap());
    let original = fs::read_to_string(std::env::args().nth(2).unwrap()).unwrap();
    for name in ["one", "two"] {
        let root = base.join(name);
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname=\"package_binding\"\nversion=\"0.1.0\"\nedition=\"2021\"\n[workspace]\n").unwrap();
        fs::write(root.join("src/lib.rs"), &original).unwrap();
    }
    let one=base.join("one"); let two=base.join("two");
    let world=RustWorld::open(&[("fixture".into(), one.as_path())], &Switches::all_on());
    let other=RustWorld::open(&[("fixture".into(), two.as_path())], &Switches::all_on());
    let proposal=world.propose_implementation(&Implement{target:"collect_values".into()}).unwrap();
    let refusal=other.apply_implementation(&proposal).unwrap_err();
    assert!(refusal.contains("another source package"));
    for root in [&one,&two] {
        assert_eq!(fs::read_to_string(root.join("src/lib.rs")).unwrap(),original);
        assert!(!root.join("Cargo.lock").exists()); assert!(!root.join("target").exists());
    }
    println!("{}; evidence: {}",refusal,proposal.evidence().display());
}
