pub fn target(xs: &[i32]) -> Vec<i32> { return xs.iter().copied().filter(|v| *v % 2 == 0).collect::<::std::vec::Vec<_>>() }
// fixture tail is immutable
#[cfg(test)] mod tests { use super::*; #[test] fn contract() { assert_eq!(target(&[2]),vec![2]); } }
