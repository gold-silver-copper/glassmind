pub fn 語(s: &str) -> Vec<String> { return s.split_whitespace().map(::std::string::String::from).collect::<::std::vec::Vec<_>>() }
// fixture tail is immutable
#[cfg(test)] mod tests { use super::*; #[test] fn contract() { assert_eq!(語(" one	 deux
三 "),vec!["三","deux","one"]); assert!(語(" 	").is_empty()); } }
