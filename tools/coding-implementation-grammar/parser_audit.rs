use std::io::{self, BufRead};
#[path = "/home/gsc/.cache/glassmind/continuous-20261010/implgrammar-run/revision1/crates/english-rust/src/implementation_grammar.rs"]
mod previous;
fn disposition<T>(value: Option<Result<T, String>>) -> String {
    match value { None => "other-route".into(), Some(Err(_)) => "refused".into(), Some(Ok(_)) => "accepted".into() }
}
fn main() {
    for text in io::stdin().lock().lines() {
        let text = text.unwrap();
        println!("{}\t{}\t{}", text, disposition(previous::request(&text)), disposition(glassmind_english_rust::implementation_grammar::request(&text)));
    }
}
