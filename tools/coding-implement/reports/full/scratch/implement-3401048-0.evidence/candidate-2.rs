pub fn 演算(s: &str) -> String { return s.split_whitespace().collect::<::std::string::String>() }
// fixture tail is immutable
#[cfg(test)] mod tests { use super::*; #[test] fn contract() { assert_eq!(演算("a é三"),"三é a"); assert_eq!(演算(""),""); } }
